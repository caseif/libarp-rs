pub enum CompressionType {
    Deflate,
}

impl CompressionType {
    pub(crate) fn get_magic(&self) -> &[u8; 2] {
        let s = match self {
            CompressionType::Deflate => "df",
        };
        assert_eq!(s.len(), 2);
        assert!(s.is_ascii());
        s.as_bytes().try_into().unwrap()
    }
}
