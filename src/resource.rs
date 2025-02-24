use std::fmt::{Display, Formatter};
use std::sync::Arc;
use crate::defines::{UID_NAMESPACE_SEPARATOR, UID_PATH_SEPARATOR};
use crate::Package;

pub struct Resource {
    pub descriptor: ResourceDescriptor,
    pub data: Vec<u8>,
}

impl Resource {
    pub fn unload(self) {
        todo!()
    }
}

#[derive(Clone)]
pub struct ResourceDescriptor {
    pub package: Arc<Package>,
    pub identifier: ResourceIdentifier,
    pub extension: String,
    pub media_type: String,
    pub size: u64,
    pub(crate) index: u32,
}

impl ResourceDescriptor {
    pub fn load(&self) -> Result<Resource, String> {
        todo!()
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

        let path_cmpts = path.split("/").map(|s| s.to_owned()).collect::<Vec<String>>();
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
