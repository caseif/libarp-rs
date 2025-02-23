use std::collections::BTreeMap;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

// input paths
const RESOURCES_DIR_REL_PATH: &str = "res";
const APACHE_MIME_TYPES_FILE_NAME: &str = "mime.types";
const SUPP_MAPPINGS_FILE_NAME: &str = "supplemental_mappings.csv";

// output paths
const GENERATED_OUT_PREFIX: &str = "generated/";
const OUT_FILE_NAME: &str = "media_types.csv";

fn main() {
    let crate_root = env::current_dir().expect("Failed to get current directory");
    let apache_mime_types_path = crate_root
        .join(RESOURCES_DIR_REL_PATH)
        .join(APACHE_MIME_TYPES_FILE_NAME);
    let supp_mappings_path = crate_root
        .join(RESOURCES_DIR_REL_PATH)
        .join(SUPP_MAPPINGS_FILE_NAME);

    if !apache_mime_types_path.exists() {
        panic!("Apache mappings file ({}) is missing", APACHE_MIME_TYPES_FILE_NAME);
    }

    if !supp_mappings_path.exists() {
        panic!("Supplemental mappings file ({}) is missing", SUPP_MAPPINGS_FILE_NAME);
    }

    println!("cargo::rerun-if-changed={}", apache_mime_types_path.display());
    println!("cargo::rerun-if-changed={}", supp_mappings_path.display());

    let apache_mappings = parse_apache_mappings(apache_mime_types_path, '\t');
    let supp_mappings = parse_csv_mappings(supp_mappings_path, ',');
    let mut combined_mappings = apache_mappings;
    combined_mappings.extend(supp_mappings);

    write_mappings_to_disk(combined_mappings);
}

fn parse_apache_mappings(path: impl AsRef<Path>, separator: char) -> BTreeMap<String, String> {
    let contents = File::open(path.as_ref())
        .and_then(|mut f| {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            Ok(s)
        })
        .expect("Failed to open Apache mappings file");
    contents.lines()
        .filter_map(|line| {
            let line = line.trim_ascii();
            if line.starts_with('#') || line.is_empty() {
                return None;
            }

            let Some(spl) = line.split_once(separator) else {
                eprintln!("Ignoring malformed line in Apache mappings file:\n    {line}");
                return None;
            };
            let mime_type = spl.0;
            let exts = spl.1.trim_ascii().split(' ');
            let submap = exts.map(|ext| (ext.to_string(), mime_type.to_string()));
            Some(submap)
        })
        .flatten()
        .collect()
}

fn parse_csv_mappings(path: impl AsRef<Path>, separator: char) -> BTreeMap<String, String> {
    let contents = File::open(path.as_ref())
        .and_then(|mut f| {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            Ok(s)
        })
        .expect("Failed to open supplemental mappings file");
    contents.lines()
        .filter_map(|line| {
            let line = line.trim_ascii();
            if line.starts_with('#') || line.is_empty() {
                return None;
            }

            let Some(spl) = line.split_once(separator) else {
                eprintln!("Ignoring malformed line in supplemental mappings file:\n    {line}");
                return None;
            };
            Some((spl.0.trim_ascii().to_string(), spl.1.trim_ascii().to_string()))
        })
        .collect()
}

fn write_mappings_to_disk(mappings: BTreeMap<String, String>) {
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gen_dir_path = out_dir_path.join(GENERATED_OUT_PREFIX);
    let out_file_path = gen_dir_path.join(OUT_FILE_NAME);

    _ = fs::create_dir_all(gen_dir_path);

    let mut out_file = File::create(out_file_path).expect("Failed to create Apache mappings file");

    for (k, v) in mappings {
        let line = format!("{},{}\n", k, v);
        out_file.write_all(line.as_bytes()).expect("Failed to write mappings");
    }
}
