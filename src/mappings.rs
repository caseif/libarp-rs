use std::collections::HashMap;

const MAPPINGS_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/generated/media_types.csv"));

pub fn load_arp_builtin_media_types() -> HashMap<&'static str, &'static str> {
    load_media_types_from_csv(MAPPINGS_CONTENT)
}

pub fn load_media_types_from_csv(content: &str) -> HashMap<&str, &str> {
    content.lines()
        .filter_map(|line| {
            let line = line.trim_ascii();
            if line.is_empty() {
                return None;
            }
            line.split_once(',')
        })
        .collect()
}
