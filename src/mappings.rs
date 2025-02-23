use std::collections::HashMap;

const MAPPINGS_CONTENT: &str = include_str!(concat!(env!("OUT_DIR"), "/generated/media_types.csv"));

pub(crate) fn load_builtin_media_types() -> HashMap<&'static str, &'static str> {
    MAPPINGS_CONTENT.lines()
        .filter_map(|line| {
            let line = line.trim_ascii();
            if line.is_empty() {
                return None;
            }
            line.split_once(',')
        })
        .collect()
}
