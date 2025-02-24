use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use std::rc::Rc;
use crate::defines::*;
use crate::{CompressionType, ResourceDescriptor, ResourceIdentifier, DEFAULT_MEDIA_TYPE};

pub struct Package {
    namespace: String,
    catalogue: LoadedCatalogue,
}

struct LoadedCatalogue {
    dirs: HashMap<u32, DirectoryNode>,
    resources: HashMap<u32, ResourceNode>,
}

struct DirectoryNode {
    index: u32,
    name: String,
    data_off: u64,
    data_len: u64,
    children: HashMap<String, u32>,
}

struct ResourceNode {
    index: u32,
    name: String,
    ext: String,
    media_type: String,
    data_part: u16,
    data_off: u64,
    data_len_packed: u64,
    data_len_unpacked: u64,
    crc: u32,
}

struct PackageMeta {
    major_version: u16,
    compression_type: Option<CompressionType>,
    namespace: String,
    total_parts: u16,
    cat_off: u64,
    cat_len: u64,
    node_count: u32,
    directory_count: u32,
    resource_count: u32,
    body_off: u64,
    body_len: u64,
}

impl Package {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Rc<Self>, String> {
        let mut file = std::fs::File::open(path.as_ref()).map_err(|e| e.to_string())?;
        load_from(&mut file, false).map_err(|e| e.to_string())
    }

    pub fn load_from_memory(data: &[u8]) -> Result<Rc<Self>, String> {
        let mut cursor = Cursor::new(data);
        load_from(&mut cursor, true).map_err(|e| e.to_string())
    }

    pub fn find_resource(self: &Rc<Self>, uid: &ResourceIdentifier)
                         -> Result<ResourceDescriptor, String> {
        if self.namespace != uid.namespace {
            return Err("Namespace does not match".to_owned());
        }

        let mut cur_dir = self.catalogue.dirs.get(&0)
            .expect("Failed to get root directory of package"); // root node
        for component in &uid.components[0..(uid.components.len() - 1)] {
            let Some(&child_index) = cur_dir.children.get(component) else {
                return Err("No resource exists with the given identifier".to_owned());
            };

            let Some(next_dir) = self.catalogue.dirs.get(&child_index) else {
                return Err("No resource exists with the given identifier".to_owned());
            };

            cur_dir = next_dir;
        }

        let resource_node_name = uid.components[uid.components.len() - 1].as_str();
        let Some(&resource_node_index) = cur_dir.children.get(resource_node_name) else {
            return Err("No resource exists with the given identifier".to_owned());
        };
        let Some(resource_node) = self.catalogue.resources.get(&resource_node_index) else {
            return Err("No resource exists with the given identifier".to_owned());
        };

        Ok(ResourceDescriptor {
            package: Rc::clone(self),
            identifier: uid.clone(),
            extension: resource_node.ext.clone(),
            media_type: resource_node.media_type.clone(),
            size: resource_node.data_len_unpacked,
            index: resource_node_index,
        })
    }

    pub fn get_all_resource_descriptors(self: &Rc<Package>) -> Vec<ResourceDescriptor> {
        let mut dir_queue = Vec::new();
        let mut resources = Vec::new();

        let root_dir = self.catalogue.dirs.get(&0)
            .expect("Failed to get root directory for package");
        dir_queue.push((root_dir, ResourceIdentifier::new(self.namespace.clone(), vec![])));
        while let Some((cur_dir, cur_uid)) = dir_queue.pop() {
            for (child_name, child_index) in &cur_dir.children {
                let child_uid = ResourceIdentifier::new(
                    cur_uid.namespace.clone(),
                    {
                        let mut c = cur_uid.components.clone();
                        c.push(child_name.clone());
                        c
                    },
                );
                if let Some(child_dir) = self.catalogue.dirs.get(&child_index) {
                    dir_queue.push((child_dir, child_uid));
                } else if let Some(child_res) = self.catalogue.resources.get(&child_index) {
                    resources.push(ResourceDescriptor {
                        package: Rc::clone(self),
                        identifier: child_uid,
                        extension: child_res.ext.clone(),
                        media_type: child_res.media_type.clone(),
                        size: child_res.data_len_unpacked,
                        index: *child_index,
                    });
                } else {
                    //TODO: shouldn't happen
                }
            }
        }

        resources
    }
}

fn load_from<R: Read + Seek>(reader: &mut R, is_in_memory: bool) -> Result<Rc<Package>, String> {
    let mut header_buf = [0u8; PACKAGE_HEADER_LEN as usize];
    reader.read_exact(&mut header_buf).map_err(|e| e.to_string())?;
    let package_meta = parse_header(&header_buf).map_err(|e| e.to_string())?;

    if package_meta.major_version != 1 {
        return Err("Unsupported format version".to_owned());
    }
    if is_in_memory && package_meta.total_parts > 0 {
        return Err("In-memory packages cannot contain multiple parts".to_owned());
    } 
    //TODO: validate header

    let mut catalogue_buf = Vec::with_capacity(package_meta.cat_len as usize);
    catalogue_buf.resize(package_meta.cat_len as usize, 0u8);
    reader.seek(std::io::SeekFrom::Start(package_meta.cat_off)).map_err(|e| e.to_string())?;
    reader.read_exact(
        catalogue_buf.as_mut_slice(),
    )
        .map_err(|e| e.to_string())?;
    let mut catalogue = parse_catalogue(
        &catalogue_buf,
        package_meta.node_count,
        package_meta.directory_count,
        package_meta.resource_count,
    ).map_err(|e| e.to_string())?;

    let node_names = catalogue.dirs.iter()
        .map(|(i, n)| (*i, n.name.clone()))
        .chain(
            catalogue.resources.iter()
                .map(|(i, n)| (*i, n.name.clone()))
        )
        .collect::<HashMap<u32, String>>();

    for (_, dir_node) in &mut catalogue.dirs {
        assert_eq!(dir_node.data_len % 4, 0);
        let mut child_indices_buf: Vec<u8> = Vec::with_capacity(dir_node.data_len as usize);
        child_indices_buf.resize(dir_node.data_len as usize, 0);
        reader.seek(std::io::SeekFrom::Start(package_meta.body_off + dir_node.data_off))
            .map_err(|e| e.to_string())?;
        reader.read_exact(&mut child_indices_buf)
            .map_err(|e| e.to_string())?;

        let child_count = dir_node.data_len as usize / size_of::<u32>();
        let child_indices: Vec<u32> = child_indices_buf.chunks(size_of::<u32>())
            .map(|buf| u32::from_le_bytes(buf.try_into().unwrap()))
            .collect();

        dir_node.children.reserve(child_count);
        for child_index in child_indices {
            dir_node.children.insert(node_names.get(&child_index).unwrap().clone(), child_index);
        }
    }

    Ok(Rc::new(Package {
        namespace: package_meta.namespace.trim().to_string(),
        catalogue,
    }))
}

fn parse_header(header: &[u8]) -> Result<PackageMeta, String> {
    let magic = &header[PACK_HEADER_MAGIC_OFF..PACK_HEADER_MAGIC_END_OFF];
    let version = read_u16_le(header, PACK_HEADER_VERSION_OFF);
    let compress_magic = &header[PACK_HEADER_COMPRESSION_OFF..PACK_HEADER_COMPRESSION_END_OFF];
    let namespace_bytes = &header[PACK_HEADER_NAMESPACE_OFF..PACK_HEADER_NAMESPACE_END_OFF].iter()
        .take_while(|c| **c != 0)
        .copied()
        .collect::<Vec<_>>();
    let namespace = String::from_utf8_lossy(namespace_bytes.as_slice());
    let total_parts = read_u16_le(header, PACK_HEADER_PARTS_OFF);
    let cat_off = read_u64_le(header, PACK_HEADER_CAT_OFF_OFF);
    let cat_len = read_u64_le(header, PACK_HEADER_CAT_LEN_OFF);
    let node_count = read_u32_le(header, PACK_HEADER_NODE_CNT_OFF);
    let dir_count = read_u32_le(header, PACK_HEADER_DIR_CNT_OFF);
    let res_count = read_u32_le(header, PACK_HEADER_RES_CNT_OFF);
    let body_off = read_u64_le(header, PACK_HEADER_BODY_OFF_OFF);
    let body_len = read_u64_le(header, PACK_HEADER_BODY_LEN_OFF);
    
    if magic != FORMAT_MAGIC {
        return Err("Format magic is incorrect".to_owned());
    }

    let compression_type = if compress_magic[0] != 0 {
        Some(match CompressionType::from_magic(compress_magic.try_into().unwrap()) {
            Some(c) => c,
            None => { return Err("Compression magic not recognized".to_owned()) }
        })
    } else {
        None
    };
    
    Ok(PackageMeta {
        major_version: version,
        compression_type,
        namespace: namespace.into_owned(),
        total_parts,
        cat_off,
        cat_len,
        node_count,
        directory_count: dir_count,
        resource_count: res_count,
        body_off,
        body_len,
    })
}

fn parse_catalogue(buf: &[u8], node_count: u32, dir_count: u32, resource_count: u32)
    -> Result<LoadedCatalogue, String> {
    let mut cursor = Cursor::new(buf);

    let mut dir_nodes = HashMap::with_capacity(dir_count as usize);
    let mut res_nodes = HashMap::with_capacity(resource_count as usize);

    for index in 0..node_count {
        let mut len_buf = [0u8; 2];
        cursor.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
        let len = u16::from_le_bytes(len_buf);

        let mut desc_buf = Vec::with_capacity(len as usize);
        desc_buf.resize(len as usize, 0u8);
        cursor.read_exact(&mut desc_buf[2..]).map_err(|e| e.to_string())?;

        let ty = desc_buf[ND_TYPE_OFF];
        let part_index = read_u16_le(&desc_buf, ND_PART_OFF);
        let data_off = read_u64_le(&desc_buf, ND_DATA_OFF_OFF);
        let packed_len = read_u64_le(&desc_buf, ND_PACKED_DATA_LEN_OFF);
        let unpacked_len = read_u64_le(&desc_buf, ND_UNPACKED_DATA_LEN_OFF);
        let crc = read_u32_le(&desc_buf, ND_CRC_OFF);
        let name_len = desc_buf[ND_NAME_LEN_OFF];
        let ext_len = desc_buf[ND_EXT_LEN_OFF];
        let mt_len = desc_buf[ND_MT_LEN_OFF];

        let mut name_buf = Vec::with_capacity(name_len as usize);
        let mut ext_buf = Vec::with_capacity(ext_len as usize);
        let mut mt_buf = Vec::with_capacity(mt_len as usize);

        name_buf.resize(name_len as usize, 0);
        ext_buf.resize(ext_len as usize, 0);
        mt_buf.resize(mt_len as usize, 0);

        let mut subcursor = Cursor::new(&desc_buf[ND_NAME_OFF..]);
        subcursor.read_exact(&mut name_buf).map_err(|e| e.to_string())?;
        subcursor.read_exact(&mut ext_buf).map_err(|e| e.to_string())?;
        subcursor.read_exact(&mut mt_buf).map_err(|e| e.to_string())?;

        let name = String::from_utf8(name_buf).map_err(|e| e.to_string())?;
        let ext = String::from_utf8(ext_buf).map_err(|e| e.to_string())?;
        let media_type = if mt_len > 0 {
            String::from_utf8(mt_buf).map_err(|e| e.to_string())?
        } else {
            DEFAULT_MEDIA_TYPE.to_owned()
        };

        match ty {
            PACK_NODE_TYPE_RESOURCE => {
                res_nodes.insert(index, ResourceNode {
                    index,
                    name,
                    ext,
                    media_type,
                    data_part: part_index,
                    data_off,
                    data_len_packed: packed_len,
                    data_len_unpacked: unpacked_len,
                    crc,
                });
            }
            PACK_NODE_TYPE_DIRECTORY => {
                dir_nodes.insert(index, DirectoryNode {
                    index,
                    name,
                    data_off,
                    data_len: packed_len,
                    children: HashMap::new(),
                });
            }
            _ => {
                return Err("Encountered unrecognized node type".to_owned());
            }
        }
    }

    Ok(LoadedCatalogue {
        dirs: dir_nodes,
        resources: res_nodes,
    })
}

fn read_u16_le(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(buf[off..(off + size_of::<u16>())].try_into().unwrap())
}

fn read_u32_le(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(buf[off..(off + size_of::<u32>())].try_into().unwrap())
}

fn read_u64_le(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(buf[off..(off + size_of::<u64>())].try_into().unwrap())
}
