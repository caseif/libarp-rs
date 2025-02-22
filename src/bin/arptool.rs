use std::env;
use std::path::PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use arp::{create_arp_from_fs, PackingOptions};

pub fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Pack(subargs) => do_pack(subargs),
        Commands::Unpack(subargs) => do_unpack(subargs),
        Commands::List(subargs) => do_list(subargs),
    }
}

fn do_pack(args: PackArgs) {
    let src_path = args.source_path.canonicalize().unwrap();
    let name = args.name.unwrap_or(src_path.file_name().unwrap().to_string_lossy().to_string());
    let namespace = args.namespace.unwrap_or_else(|| name.clone());
    let max_part_len = args.part_size;
    let compression_type = match args.compression_type {
        Some(CompressionType::None) | None => None,
        Some(CompressionType::Deflate) => Some("df"),
    };
    let media_types_path = args.mappings;
    let dest_path = args.output_dir.unwrap_or(env::current_dir().unwrap());

    let opts = PackingOptions::new_v1(
        name,
        namespace,
        max_part_len,
        compression_type,
        media_types_path,
    ).unwrap();
    create_arp_from_fs(&src_path, &dest_path, opts).unwrap();
}

fn do_unpack(_args: UnpackArgs) {
    todo!()
}

fn do_list(_args: ListArgs) {
    todo!()
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Pack(PackArgs),
    Unpack(UnpackArgs),
    List(ListArgs),
}

#[derive(Args)]
struct PackArgs {
    #[arg(value_name = "directory")]
    source_path: PathBuf,
    #[arg(short = 'c', long = "compress", value_name = "type", default_value = None)]
    compression_type: Option<CompressionType>,
    #[arg(long = "deflate")]
    deflate: bool,
    #[arg(short = 'f', long = "name", value_name = "name")]
    name: Option<String>,
    #[arg(short = 'm', long = "mappings", value_name = "file")]
    mappings: Option<PathBuf>,
    #[arg(short = 'n', long = "namespace", value_name = "namespace")]
    namespace: Option<String>,
    #[arg(short = 'o', long = "output", value_name = "directory")]
    output_dir: Option<PathBuf>,
    #[arg(short = 'p', long = "part-size", value_name = "size")]
    part_size: Option<u64>,
}

#[derive(Args)]
struct UnpackArgs {
    #[arg(value_name = "ARP file")]
    source_path: PathBuf,
    #[arg(short = 'o', long = "output", value_name = "directory")]
    output: Option<PathBuf>,
    #[arg(short = 'r', long = "resource", value_name = "UID")]
    resource_path: Option<String>,
}

#[derive(Args)]
struct ListArgs {
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CompressionType {
    None,
    Deflate,
}
