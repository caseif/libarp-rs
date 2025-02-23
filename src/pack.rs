use std::collections::{HashMap, VecDeque};
use std::{env, fs, io};
use std::fs::{File, FileType};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use miniz_oxide::deflate;
use miniz_oxide::deflate::CompressionLevel;
use uuid::Uuid;
use crate::defines::*;
use crate::util::crc32c::crc32c;
use crate::util::uid::validate_path_component;

pub use crate::defines::COMPRESS_MAGIC_DEFLATE;
use crate::mappings::load_builtin_media_types;

pub const DEFAULT_MEDIA_TYPE: &str = "application/octet-stream";

pub struct PackingOptions {
    version: u16,
    name: String,
    namespace: String,
    max_part_len: Option<u64>,
    compression_type: Option<String>,
    media_types_path: Option<PathBuf>,
}

impl PackingOptions {
    pub fn new_v1(
        name: impl Into<String>,
        namespace: impl Into<String>,
        max_part_len: Option<u64>,
        compression_type: Option<impl Into<String>>,
        media_types_path: Option<impl AsRef<Path>>,
    ) -> Result<PackingOptions, String> {
        let name = name.into();
        let namespace = namespace.into();
        let compression_type = compression_type.map(|s| s.into());
        let media_types_path = media_types_path.map(|p| p.as_ref().to_path_buf());

        if name.is_empty() {
            return Err("Package name cannot be empty".to_owned());
        }

        if namespace.is_empty() {
            return Err("Package namespace cannot be empty".to_owned());
        }
        if namespace.len() as u64 > NAMESPACE_MAX_LEN {
            return Err("Package namespace is too long".to_owned());
        }
        validate_path_component(&namespace)?;

        if max_part_len.is_some_and(|len| len < PART_LEN_MIN) {
            return Err("Max part length is too small".to_owned());
        }

        let compress_magic = if let Some(compression) = compression_type {
            Some(match compression.as_str() {
                COMPRESS_MAGIC_DEFLATE => COMPRESS_TYPE_DEFLATE.to_owned(),
                _ => { return Err("Unrecognized compression type".to_owned()); }
            })
        } else { None };

        Ok(Self {
            version: 1,
            name,
            namespace,
            max_part_len,
            compression_type: compress_magic,
            media_types_path,
        })
    }
}

pub fn create_arp_from_fs(
    src_path: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
    options: PackingOptions,
) -> Result<(), String> {
    let media_types_builtin = load_builtin_media_types();

    let mut user_mappings_content = String::new();
    let media_types_user = if let Some(mt_path) = &options.media_types_path {
        let mut mt_file = File::open(mt_path).map_err(|e| e.to_string())?;
        mt_file.read_to_string(&mut user_mappings_content).map_err(|e| e.to_string())?;
        load_media_types(user_mappings_content.as_str())?
    } else {
        Default::default()
    };

    let mut all_media_types = media_types_builtin;
    all_media_types.extend(media_types_user);

    let nodes = traverse_fs(src_path.as_ref(), &all_media_types)?;

    write_package_to_disk(nodes, target_dir.as_ref(), &options)
}

struct FsNode {
    ty: FileType,
    target_path: PathBuf,
    media_type: String,
    size: u64,
    index: u32,
    child_dir_indices: Vec<u32>,
    child_dir_paths: Vec<PathBuf>,
    child_file_indices: Vec<u32>,
    child_file_paths: Vec<PathBuf>,
}

struct ProcessedNodeData {
    data: Vec<u8>,
    crc: u32,
}

fn traverse_fs(
    root_path: impl AsRef<Path>,
    media_types: &HashMap<&str, &str>
) -> Result<Vec<FsNode>, String> {

    let mut dir_queue: VecDeque<PathBuf> = VecDeque::from([root_path.as_ref().to_owned()]);
    let mut file_queue: VecDeque<PathBuf> = VecDeque::new();

    let mut dir_nodes: Vec<FsNode> = Vec::new();
    let mut file_nodes: Vec<FsNode> = Vec::new();

    let mut dir_index_map: HashMap<PathBuf, u32> = HashMap::new();
    let mut file_index_map: HashMap<PathBuf, u32> = HashMap::new();

    let mut cur_index = 0;

    while let Some(dir_path) = dir_queue.pop_front() {
        let meta = dir_path.metadata().map_err(|err| err.to_string())?;

        let ty = meta.file_type();
        if !ty.is_dir() && !ty.is_file() {
            return Err("Only regular files and directories are supported".to_owned());
        }

        let media_type = if meta.is_file() {
            dir_path.extension()
                .and_then(|ext|
                    media_types.get(ext.to_string_lossy().as_ref()).copied()
                )
                .unwrap_or(DEFAULT_MEDIA_TYPE)
        } else {
            ""
        };

        let size = meta.len();

        let mut child_dir_paths = vec![];
        let mut child_file_paths = vec![];

        if meta.is_dir() {
            let dir = dir_path.read_dir().map_err(|err| err.to_string())?;
            for child in dir {
                let child = child.map_err(|err| err.to_string())?;
                let child_meta = child.metadata().map_err(|err| err.to_string())?;
                if child_meta.is_dir() {
                    dir_queue.push_back(child.path());
                    child_dir_paths.push(child.path());
                } else if child_meta.is_file() {
                    file_queue.push_back(child.path());
                    child_file_paths.push(child.path());
                } else {
                    println!(
                        "Warning: Ignoring child '{}' as it is not a directory or regular file",
                        child.path().display(),
                    );
                }
            }
        }

        let node = FsNode {
            ty,
            target_path: dir_path.clone(),
            media_type: media_type.to_owned(),
            size,
            index: cur_index,
            child_dir_indices: Vec::with_capacity(child_dir_paths.len()),
            child_dir_paths,
            child_file_indices: Vec::with_capacity(child_file_paths.len()),
            child_file_paths,
        };
        dir_nodes.push(node);
        dir_index_map.insert(dir_path, cur_index);
        cur_index += 1;
    }

    while let Some(file_path) = file_queue.pop_front() {
        let meta = file_path.metadata().map_err(|err| err.to_string())?;

        let ty = meta.file_type();
        if !ty.is_dir() && !ty.is_file() {
            return Err("Only regular files and directories are supported".to_owned());
        }

        let media_type = file_path.extension()
            .and_then(|ext|
                media_types.get(ext.to_string_lossy().as_ref()).copied()
            )
            .unwrap_or(DEFAULT_MEDIA_TYPE);

        let size = meta.len();

        let node = FsNode {
            ty,
            target_path: file_path.clone(),
            media_type: media_type.to_owned(),
            size,
            index: cur_index,
            child_dir_indices: vec![],
            child_dir_paths: vec![],
            child_file_indices: vec![],
            child_file_paths: vec![],
        };
        file_nodes.push(node);
        file_index_map.insert(file_path, cur_index);
        cur_index += 1;
    }

    for node in &mut dir_nodes {
        for child_path in &mut node.child_dir_paths {
            node.child_dir_indices.push(dir_index_map[child_path]);
        }
        for child_path in &mut node.child_file_paths {
            node.child_file_indices.push(file_index_map[child_path]);
        }

        node.child_dir_paths.clear();
        node.child_file_paths.clear();
    }

    let final_nodes = dir_nodes.into_iter().chain(file_nodes.into_iter()).collect();
    Ok(final_nodes)
}

fn load_media_types(csv_contents: &str) -> Result<HashMap<&str, &str>, String> {
    let mappings = csv_contents.lines()
        .filter_map(|line| {
            let spl = line.split_once(",")?;
            Some((spl.0, spl.1))
        })
        .collect::<HashMap<&str, &str>>();

    Ok(mappings)
}

fn write_package_to_disk(
    nodes: Vec<FsNode>,
    target_dir: impl AsRef<Path>,
    options: &PackingOptions
) -> Result<(), String> {
    let node_count = nodes.len();
    let dir_count = nodes.iter().filter(|n| n.ty.is_dir()).count();
    let resource_count = nodes.iter().filter(|n| n.ty.is_file()).count();

    let catalogue_len = compute_catalogue_len(&nodes);

    let catalogue_path = env::temp_dir().join(Uuid::new_v4().to_string());
    let mut catalogue_file = File::create_new(catalogue_path).map_err(|e| e.to_string())?;

    // generate temp file for new part
    let part_1_path = env::temp_dir().join(Uuid::new_v4().to_string());
    // open new part file
    let mut part_1_file = File::create_new(&part_1_path).map_err(|e| e.to_string())?;
    // reserve bytes at start so we can populate the header later
    part_1_file.seek(SeekFrom::Start(PACKAGE_HEADER_LEN + catalogue_len))
        .map_err(|e| e.to_string())?;

    let mut part_paths: Vec<PathBuf> = vec![part_1_path.clone()];
    let mut part_body_lens: Vec<u64> = Vec::new();
    let mut cur_part_body_len: u64 = 0;

    let mut cur_part_file = part_1_file;

    let mut cur_part = 1;
    for node in nodes {
        let processed_data = load_node_data(&node, &options)?;

        let node_len = processed_data.data.len() as u64;

        let new_part_body_len = cur_part_body_len + node_len;
        let new_part_len = new_part_body_len + PACKAGE_PART_HEADER_LEN;
        if options.max_part_len.is_some_and(|max_len| new_part_len > max_len) {
            if cur_part_body_len == 0 {
                return Err("Max part size is smaller than largest resource".to_owned());
            }

            // part is finalized - push its length
            part_body_lens.push(cur_part_body_len);
            // advance to the next part
            cur_part += 1;
            // reset body length for new part
            cur_part_body_len = 0;

            // generate temp file for new part
            let cur_part_path = env::temp_dir().join(Uuid::new_v4().to_string());
            part_paths.push(cur_part_path.clone());
            // open new part file
            cur_part_file = File::create_new(cur_part_path).map_err(|e| e.to_string())?;

            // populate part header
            let mut part_header_buf: Vec<u8> = Vec::with_capacity(PACKAGE_PART_HEADER_LEN as usize);
            // part format magic
            part_header_buf.extend_from_slice(&PART_MAGIC);
            // part index
            push_u16_le(&mut part_header_buf, cur_part);
            // extend to full part header length (last section is reserved)
            part_header_buf.resize(0x10, 0u8);
            // write header to file
            cur_part_file.write_all(part_header_buf.as_slice()).map_err(|e| e.to_string())?;

            if cur_part > PARTS_MAX {
                return Err("Part count would exceed maximum".to_owned());
            }
        }

        let type_ordinal = if node.ty.is_file() {
            PACK_NODE_TYPE_RESOURCE
        } else if node.ty.is_dir() {
            PACK_NODE_TYPE_DIRECTORY
        } else {
            panic!("Unhandled FS node type");
        };

        let name = if node.index == 0 {
            &[]
        } else {
            node.target_path.file_stem().unwrap().as_encoded_bytes()
        };
        let ext = if node.ty.is_file() {
            node.target_path.extension().map(|ext| ext.as_encoded_bytes()).unwrap_or(&[])
        } else {
            &[]
        };
        let name_len = name.len();
        let ext_len = ext.len();
        let media_type_len = node.media_type.len();
        assert!(name_len <= u8::MAX as usize);
        assert!(ext_len <= u8::MAX as usize);
        assert!(media_type_len <= u8::MAX as usize);

        // write node contents to part file
        cur_part_file.write_all(&processed_data.data).map_err(|e| e.to_string())?;

        // build node descriptor in memory
        let node_desc_len = compute_node_desc_len(&node);
        let mut node_desc: Vec<u8> = Vec::with_capacity(node_desc_len as usize);
        push_u16_le(&mut node_desc, node_desc_len);
        node_desc.push(type_ordinal);
        push_u16_le(&mut node_desc, cur_part);
        push_u64_le(&mut node_desc, cur_part_body_len);
        push_u64_le(&mut node_desc, processed_data.data.len() as u64);
        push_u64_le(&mut node_desc, node.size);
        push_u32_le(&mut node_desc, processed_data.crc);
        node_desc.push(name_len as u8);
        node_desc.push(ext_len as u8);
        node_desc.push(media_type_len as u8);
        node_desc.extend_from_slice(name);
        node_desc.extend_from_slice(ext);
        node_desc.extend_from_slice(node.media_type.as_bytes());
        assert_eq!(node_desc.len(), node_desc_len as usize);

        // write node descriptor to disk
        catalogue_file.write_all(&node_desc).map_err(|e| e.to_string())?;

        cur_part_body_len += processed_data.data.len() as u64;
    }

    // store body length of final part
    part_body_lens.push(cur_part_body_len);

    // flush and rewind completed catalogue file
    catalogue_file.flush().map_err(|e| e.to_string())?;
    catalogue_file.rewind().map_err(|e| e.to_string())?;

    let total_parts = cur_part;

    let mut part_1_file = if total_parts == 1 {
        // still on part 1, reuse the handle
        cur_part_file
    } else {
        // close current part
        _ = cur_part_file;
        // open the first part again
        File::options()
            .write(true)
            .truncate(false)
            .open(part_1_path)
            .map_err(|e| e.to_string())?
    };

    let mut header_buf: Vec<u8> = Vec::with_capacity(PACKAGE_HEADER_LEN as usize);
    // format magic
    header_buf.extend_from_slice(&FORMAT_MAGIC);
    // version
    push_u16_le(&mut header_buf, options.version);
    // compression type
    let compression_type_buf = match options.compression_type.as_ref() {
        Some(compression_type) => {
            assert!(compression_type.is_ascii());
            &compression_type.as_bytes()[0..PACK_HEADER_COMPRESSION_LEN]
        }
        None => &[0u8; 2],
    };
    header_buf.extend_from_slice(compression_type_buf);
    // namespace
    let mut namespace_buf = Vec::with_capacity(PACK_HEADER_NAMESPACE_LEN);
    namespace_buf.extend_from_slice(options.namespace.as_bytes());
    namespace_buf.resize(PACK_HEADER_NAMESPACE_LEN, 0u8);
    header_buf.extend_from_slice(&namespace_buf);
    // total parts count
    push_u16_le(&mut header_buf, total_parts);
    // catalogue offset
    push_u64_le(&mut header_buf, PACKAGE_HEADER_LEN);
    // catalogue length
    push_u64_le(&mut header_buf, catalogue_len);
    // node count
    push_u32_le(&mut header_buf, node_count as u32);
    // directory count
    push_u32_le(&mut header_buf, dir_count as u32);
    // resource count
    push_u32_le(&mut header_buf, resource_count as u32);
    // body offset
    push_u64_le(&mut header_buf, PACKAGE_HEADER_LEN + catalogue_len);
    // body length
    push_u64_le(&mut header_buf, part_body_lens[0]);

    assert_eq!(header_buf.len(), PACK_HEADER_BODY_LEN_END_OFF);
    // extend to full header length (last section is reserved)
    header_buf.resize(0x100, 0u8);

    // write package header
    part_1_file.rewind().map_err(|e| e.to_string())?;
    part_1_file.write_all(header_buf.as_slice()).map_err(|e| e.to_string())?;
    // copy catalogue contents
    io::copy(&mut catalogue_file, &mut part_1_file).map_err(|e| e.to_string())?;
    part_1_file.flush().map_err(|e| e.to_string())?;

    // release part 1 file handle
    _ = part_1_file;

    let target_dir_ref = target_dir.as_ref();

    if !target_dir_ref.exists() {
        fs::create_dir(target_dir_ref).map_err(|e| e.to_string())?;
    }

    // copy temp files to final paths
    for i in 0..total_parts {
        let src = &part_paths[i as usize];
        let dest = if i == 0 && total_parts == 1 {
            target_dir_ref.join(format!("{}.arp", options.name))
        } else {
            target_dir_ref.join(format!("{}.part{:0>3}.arp", options.name, i + 1))
        };
        fs::copy(src, dest).map_err(|e| e.to_string())?;
        fs::remove_file(src).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn load_node_data(node: &FsNode, options: &PackingOptions)
                  -> Result<ProcessedNodeData, String> {
    let data: Vec<u8> = if node.ty.is_file() {
        let mut data = Vec::new();

        let mut file = File::open(&node.target_path).map_err(|e| e.to_string())?;
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;

        if let Some(compression) = options.compression_type.as_ref() {
            match compression.as_str() {
                COMPRESS_TYPE_DEFLATE =>
                    deflate::compress_to_vec(&data, CompressionLevel::BestCompression as u8),
                _ => panic!("Unhandled compression type {}", compression),
            }
        } else {
            data
        }
    } else if node.ty.is_dir() {
        node.child_dir_indices.iter()
            .chain(node.child_file_indices.iter())
            .map(|idx| idx.to_le_bytes())
            .flatten()
            .collect()
    } else {
        panic!("Unhandled node type {:?}", node.ty);
    };

    let crc = crc32c(&data);

    Ok(ProcessedNodeData {
        data,
        crc,
    })
}

fn compute_node_desc_len(node: &FsNode) -> u16 {
    let stem_len = if node.index == 0 {
        0
    } else {
        node.target_path.file_stem().unwrap().as_encoded_bytes().len()
    };
    let ext_len = node.target_path.extension()
        .map(|ext| ext.as_encoded_bytes().len())
        .unwrap_or(0);
    let media_type_len = node.media_type.len();

    (NODE_DESC_BASE_LEN + (stem_len + ext_len + media_type_len) as u64) as u16
}

fn compute_catalogue_len(nodes: &Vec<FsNode>) -> u64 {
    let catalogue_base_len: u64 = nodes.len() as u64 * NODE_DESC_BASE_LEN;
    let mut catalogue_len: u64 = catalogue_base_len;

    for node in nodes {
        catalogue_len += compute_node_desc_len(node) as u64;
    }

    catalogue_len
}

#[inline(always)]
fn push_u16_le(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&val.to_le_bytes());
}

#[inline(always)]
fn push_u32_le(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_le_bytes());
}

#[inline(always)]
fn push_u64_le(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&val.to_le_bytes());
}
