use std::fmt::{Display, Formatter};
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use miniz_oxide::inflate::stream::{inflate, InflateState};
use miniz_oxide::{DataFormat, MZFlush, MZStatus};
use crate::defines::{PACKAGE_PART_HEADER_LEN, UID_NAMESPACE_SEPARATOR, UID_PATH_SEPARATOR};
use crate::{CompressionType, Package};
use crate::util::crc32c::crc32c;

pub struct Resource {
    pub descriptor: ResourceDescriptor,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct ResourceDescriptor {
    pub package: Arc<Package>,
    pub identifier: ResourceIdentifier,
    pub name: String,
    pub extension: String,
    pub media_type: String,
    pub size: u64,
    pub(crate) index: u32,
}

impl ResourceDescriptor {
    pub fn load(&self) -> Result<Vec<u8>, String> {
        let resource = self.package.catalogue.resources.get(&self.index).unwrap();
        let data_off = if resource.data_part == 1 {
            self.package.meta.body_off + resource.data_off
        } else {
            PACKAGE_PART_HEADER_LEN + resource.data_off
        };
        let data_len_packed = resource.data_len_packed;

        let resource_data = if let Some(mem_buffer) = self.package.mem_buffer {
            assert!(self.package.part_files.is_none());
            assert_eq!(resource.data_part, 1);

            Vec::from(
                &mem_buffer[(data_off as usize)..((data_off + data_len_packed) as usize)]
            )
        } else if let Some(part_files) = self.package.part_files.as_ref() {
            let mut part_files_borrowed = part_files.write().unwrap();
            let part_file = &mut part_files_borrowed[resource.data_part as usize - 1];

            let mut buf = Vec::with_capacity(data_len_packed as usize);
            buf.resize(data_len_packed as usize, 0);
            part_file.seek(SeekFrom::Start(data_off)).unwrap();
            part_file.read_exact(&mut buf).map_err(|e| e.to_string())?;

            buf
        } else {
            panic!("Memory buffer or part file list must be populated");
        };

        let actual_crc = crc32c(&resource_data);
        if actual_crc != resource.crc {
            return Err("CRC mismatch".to_owned());
        }

        let resource_data = match self.package.meta.compression_type.as_ref() {
            Some(CompressionType::Deflate) => {
                let mut inflate_state = InflateState::new(DataFormat::Zlib);
                let mut unpacked_data = Vec::with_capacity(resource.data_len_unpacked as usize);
                let mut remaining_bytes = resource.data_len_unpacked as usize;
                let mut output_buf = [0u8; 4096];
                while remaining_bytes > 0 {
                    let result = inflate(
                        &mut inflate_state,
                        &resource_data,
                        &mut output_buf,
                        MZFlush::None,
                    );

                    let status = result.status.map_err(|e| format!("{:?}", e))?;
                    remaining_bytes -= result.bytes_written;
                    if remaining_bytes == 0 && status != MZStatus::StreamEnd {
                        return Err("Expected end of DEFLATE stream".to_owned());
                    } else if remaining_bytes > 0 && status == MZStatus::StreamEnd {
                        return Err("Encountered premature end of DEFLATE stream".to_owned());
                    }

                    unpacked_data.extend_from_slice(&output_buf[0..result.bytes_written]);
                }

                unpacked_data
            }
            None => resource_data,
        };

        Ok(resource_data)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceIdentifier {
    pub namespace: String,
    pub components: Vec<String>,
}

impl ResourceIdentifier {
    pub fn new(namespace: impl Into<String>, components: Vec<String>) -> Self {
        Self { namespace: namespace.into(), components, }
    }

    pub fn parse(s: impl AsRef<str>) -> Result<Self, String> {
        let s_ref = s.as_ref();

        let Some((ns, path)) = s_ref.split_once(UID_NAMESPACE_SEPARATOR) else {
            return Err("Resource UID must contain namespace".to_owned());
        };

        if path.contains(UID_NAMESPACE_SEPARATOR) {
            return Err("Resource UID contains more than once namespace separator".to_owned());
        }

        if ns.contains(UID_PATH_SEPARATOR) {
            return Err("Resource UID namespace cannot contain path separator".to_owned());
        }

        let path_cmpts = path.split(UID_PATH_SEPARATOR)
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        if path_cmpts.iter().any(|s| s.is_empty()) {
            return Err("Resource UID contains empty path component".to_owned());
        }

        Ok(Self { namespace: ns.to_owned(), components: path_cmpts })
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}{}{}",
            self.namespace,
            UID_NAMESPACE_SEPARATOR,
            self.components.join(UID_PATH_SEPARATOR.to_string().as_str())
        )
    }
}

impl Display for ResourceIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
