const MAGIC_DEFLATE: &str = "df";

pub enum CompressionType {
    Deflate,
}

impl CompressionType {
    pub(crate) fn from_magic(magic: &[u8; 2]) -> Option<CompressionType> {
        let magic_str = String::from_utf8_lossy(magic);
        match magic_str.as_ref() {
            MAGIC_DEFLATE => Some(CompressionType::Deflate),
            _ => None,
        }
    }
    
    pub(crate) fn get_magic(&self) -> &[u8; 2] {
        let s = match self {
            CompressionType::Deflate => MAGIC_DEFLATE,
        };
        assert_eq!(s.len(), 2);
        assert!(s.is_ascii());
        s.as_bytes().try_into().unwrap()
    }
}
