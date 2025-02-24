#![allow(dead_code)]

// magic values
pub(crate) const FORMAT_MAGIC: [u8; 8] = [0x1B, 0x41, 0x52, 0x47, 0x55, 0x53, 0x52, 0x50];
pub(crate) const PART_MAGIC: [u8; 8] = [0x1B, 0x41, 0x52, 0x47, 0x55, 0x53, 0x50, 0x54];

pub const COMPRESS_TYPE_DEFLATE: &str = "deflate";
pub(crate) const COMPRESS_MAGIC_DEFLATE: &str = "df";

pub(crate) const PACKAGE_PART_1_SUFFIX: &str = ".part001";

pub(crate) const PACK_NODE_TYPE_RESOURCE: u8 = 0;
pub(crate) const PACK_NODE_TYPE_DIRECTORY: u8 = 1;

// package header constants
pub(crate) const PACKAGE_HEADER_LEN: u64 = 0x100;

pub(crate) const PACK_HEADER_MAGIC_LEN: usize = 8;
pub(crate) const PACK_HEADER_VERSION_LEN: usize = 2;
pub(crate) const PACK_HEADER_COMPRESSION_LEN: usize = 2;
pub(crate) const PACK_HEADER_NAMESPACE_LEN: usize = 48;
pub(crate) const PACK_HEADER_PARTS_LEN: usize = 2;
pub(crate) const PACK_HEADER_CAT_OFF_LEN: usize = 8;
pub(crate) const PACK_HEADER_CAT_LEN_LEN: usize = 8;
pub(crate) const PACK_HEADER_NODE_CNT_LEN: usize = 4;
pub(crate) const PACK_HEADER_DIR_CNT_LEN: usize = 4;
pub(crate) const PACK_HEADER_RES_CNT_LEN: usize = 4;
pub(crate) const PACK_HEADER_BODY_OFF_LEN: usize = 8;
pub(crate) const PACK_HEADER_BODY_LEN_LEN: usize = 8;
pub(crate) const PACK_HEADER_RESERVED_1_LEN: usize = 0x96;

pub(crate) const PACK_HEADER_MAGIC_OFF: usize = 0x00;
pub(crate) const PACK_HEADER_VERSION_OFF: usize = 0x08;
pub(crate) const PACK_HEADER_COMPRESSION_OFF: usize = 0x0A;
pub(crate) const PACK_HEADER_NAMESPACE_OFF: usize = 0x0C;
pub(crate) const PACK_HEADER_PARTS_OFF: usize = 0x3C;
pub(crate) const PACK_HEADER_CAT_OFF_OFF: usize = 0x3E;
pub(crate) const PACK_HEADER_CAT_LEN_OFF: usize = 0x46;
pub(crate) const PACK_HEADER_NODE_CNT_OFF: usize = 0x4E;
pub(crate) const PACK_HEADER_DIR_CNT_OFF: usize = 0x52;
pub(crate) const PACK_HEADER_RES_CNT_OFF: usize = 0x56;
pub(crate) const PACK_HEADER_BODY_OFF_OFF: usize = 0x5A;
pub(crate) const PACK_HEADER_BODY_LEN_OFF: usize = 0x62;
pub(crate) const PACK_HEADER_RESERVED_1_OFF: usize = 0x6A;

pub(crate) const PACK_HEADER_MAGIC_END_OFF: usize =
    PACK_HEADER_MAGIC_OFF + PACK_HEADER_MAGIC_LEN;
pub(crate) const PACK_HEADER_VERSION_END_OFF: usize =
    PACK_HEADER_VERSION_OFF + PACK_HEADER_VERSION_LEN;
pub(crate) const PACK_HEADER_COMPRESSION_END_OFF: usize =
    PACK_HEADER_COMPRESSION_OFF + PACK_HEADER_COMPRESSION_LEN;
pub(crate) const PACK_HEADER_NAMESPACE_END_OFF: usize =
    PACK_HEADER_NAMESPACE_OFF + PACK_HEADER_NAMESPACE_LEN;
pub(crate) const PACK_HEADER_PARTS_END_OFF: usize =
    PACK_HEADER_PARTS_OFF + PACK_HEADER_PARTS_LEN;
pub(crate) const PACK_HEADER_CAT_OFF_END_OFF: usize =
    PACK_HEADER_CAT_OFF_OFF + PACK_HEADER_CAT_OFF_LEN;
pub(crate) const PACK_HEADER_CAT_LEN_END_OFF: usize =
    PACK_HEADER_CAT_LEN_OFF + PACK_HEADER_CAT_LEN_LEN;
pub(crate) const PACK_HEADER_NODE_CNT_END_OFF: usize =
    PACK_HEADER_NODE_CNT_OFF + PACK_HEADER_NODE_CNT_LEN;
pub(crate) const PACK_HEADER_DIR_CNT_END_OFF: usize =
    PACK_HEADER_DIR_CNT_OFF + PACK_HEADER_DIR_CNT_LEN;
pub(crate) const PACK_HEADER_RES_CNT_END_OFF: usize =
    PACK_HEADER_RES_CNT_OFF + PACK_HEADER_RES_CNT_LEN;
pub(crate) const PACK_HEADER_BODY_OFF_END_OFF: usize =
    PACK_HEADER_BODY_OFF_OFF + PACK_HEADER_BODY_OFF_LEN;
pub(crate) const PACK_HEADER_BODY_LEN_END_OFF: usize =
    PACK_HEADER_BODY_LEN_OFF + PACK_HEADER_BODY_LEN_LEN;
pub(crate) const PACK_HEADER_RESERVED_1_END_OFF: usize =
    PACK_HEADER_RESERVED_1_OFF + PACK_HEADER_RESERVED_1_LEN;

// part header constants
pub(crate) const PACKAGE_PART_HEADER_LEN: u64 = 0x10;

pub(crate) const PART_INDEX_LEN: usize = 2;
pub(crate) const PART_UNUSED_LEN: usize = 6;

pub(crate) const PART_MAGIC_OFF: usize = 0;
pub(crate) const PART_INDEX_OFF: usize = 8;
pub(crate) const PART_UNUSED_OFF: usize = 10;

// node structure constants
pub(crate) const ND_LEN_LEN: usize = 2;
pub(crate) const ND_TYPE_LEN: usize = 1;
pub(crate) const ND_PART_LEN: usize = 2;
pub(crate) const ND_DATA_OFF_LEN: usize = 8;
pub(crate) const ND_PACKED_DATA_LEN_LEN: usize = 8;
pub(crate) const ND_UNPACKED_DATA_LEN_LEN: usize = 8;
pub(crate) const ND_CRC_LEN: usize = 4;
pub(crate) const ND_NAME_LEN_LEN: usize = 1;
pub(crate) const ND_EXT_LEN_LEN: usize = 1;
pub(crate) const ND_MT_LEN_LEN: usize = 1;

pub(crate) const ND_LEN_OFF: usize = 0x00;
pub(crate) const ND_TYPE_OFF: usize = 0x02;
pub(crate) const ND_PART_OFF: usize = 0x03;
pub(crate) const ND_DATA_OFF_OFF: usize = 0x05;
pub(crate) const ND_PACKED_DATA_LEN_OFF: usize = 0x0D;
pub(crate) const ND_UNPACKED_DATA_LEN_OFF: usize = 0x15;
pub(crate) const ND_CRC_OFF: usize = 0x1D;
pub(crate) const ND_NAME_LEN_OFF: usize = 0x21;
pub(crate) const ND_EXT_LEN_OFF: usize = 0x22;
pub(crate) const ND_MT_LEN_OFF: usize = 0x23;
pub(crate) const ND_NAME_OFF: usize = 0x24;

pub(crate) const NODE_DESC_BASE_LEN: usize = ND_NAME_OFF;

pub(crate) const NODE_NAME_MAX_LEN: usize = 0xFF;
pub(crate) const NODE_EXT_MAX_LEN: usize = 0xFF;
pub(crate) const NODE_MT_MAX_LEN: usize = 0xFF;
pub(crate) const NODE_DESC_MAX_LEN: usize =
    NODE_DESC_BASE_LEN + NODE_NAME_MAX_LEN + NODE_EXT_MAX_LEN + NODE_MT_MAX_LEN;

// the length of an index to a node descriptor
// directory nodes contain an array of node descriptor indices in their body
pub(crate) const NODE_DESC_INDEX_LEN: usize = 4;

// limits
pub(crate) const NAMESPACE_MAX_LEN: u64 = 48;
pub(crate) const PART_LEN_MIN: u64 = 4096;
pub(crate) const PARTS_MAX: u16 = 999;

// we need _some_ sane limit
pub(crate) const DIRECTORY_CONTENT_MAX_LEN: u64 = 4294967296 * NODE_DESC_INDEX_LEN as u64;

pub(crate) const UID_NAMESPACE_SEPARATOR: char = ':';
pub(crate) const UID_PATH_SEPARATOR: char = '/';
