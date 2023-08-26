use std::path::PathBuf;
use clap::Parser;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    l1_config: Option<PathBuf>,
    #[clap(short, long)]
    l2_config: Option<PathBuf>,
    #[clap(short, long)]
    script_path: Option<PathBuf>,
    #[clap(short, long)]
    dest: Option<PathBuf>,
}

#[derive(Debug, PartialEq, EnumString, Serialize, Deserialize)]
enum CpuMode {
    #[strum(serialize = "custom")]
    #[serde(rename = "custom")]
    Custom,
    #[strum(serialize = "host-passthrough")]
    #[serde(rename = "host-passthrough")]
    HostPassthrough,
    #[strum(serialize = "host-model")]
    #[serde(rename = "host-model")]
    HostModel,
    #[strum(serialize = "maximum")]
    #[serde(rename = "maximum")]
    Maximum,
}

#[derive(Debug, Serialize, Deserialize)]
struct L1VagrantConfig {
    host_name: String,
    cpus: u32,
    memory: u64,
    cpu_mode: CpuMode,
    network_interface: Option<String>,
    l2_vagrant_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct L2VagrantConfig {
    host_name: String,
    cpus: u32,
    memory: u64,
    cpu_mode: CpuMode,
    enable_network_bridge: bool,
}

impl Default for L1VagrantConfig {
    fn default() -> Self {
        Self {
            host_name: "l1-vagrant".to_string(),
            cpus: 2,
            memory: 4096,
            cpu_mode: CpuMode::Custom,
            network_interface: None,
            l2_vagrant_dir: None,
        }
    }
}

impl Default for L2VagrantConfig {
    fn default() -> Self {
        Self {
            host_name: "l2-vagrant".to_string(),
            cpus: 2,
            memory: 2048,
            cpu_mode: CpuMode::Custom,
            enable_network_bridge: false,
        }
    }
}

fn main() {
    let args = Args::parse();
    let project_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let l1_vagrant_template_path = project_path.join("resources").join("l1-vagrant-template");
    let dest_base_dir = if let Some(dest) = args.dest {
        dest
    } else {
        std::env::current_dir().unwrap()
    };
    let l1_vagrant_dest = dest_base_dir.join("l1-vagrant");
    println!("{:?}", l1_vagrant_template_path);
    println!("{:?}", l1_vagrant_template_path);
    let mut l1_vagrant_config = if let Some(config_path) = args.l1_config {
        let config_file = std::fs::File::open(config_path).unwrap();
        serde_yaml::from_reader(config_file).unwrap()
    } else {
        L1VagrantConfig::default()
    };
    fs_extra::dir::create_all(l1_vagrant_dest.as_path(), false).unwrap();
    fs_extra::dir::copy(
        l1_vagrant_template_path.as_path(),
        l1_vagrant_dest.as_path(),
        &fs_extra::dir::CopyOptions::new().content_only(true),
    ).unwrap();
    if l1_vagrant_config.l2_vagrant_dir.is_none() {
        let l2_vagrant_dest = std::env::current_dir().unwrap().join("l2-vagrant");
        let l2_vagrant_template_path = project_path.join("resources").join("l2-vagrant-template");
        fs_extra::dir::create_all(l2_vagrant_dest.as_path(), false).unwrap();
        fs_extra::dir::copy(
            l2_vagrant_template_path.as_path(),
            l2_vagrant_dest.as_path(),
            &fs_extra::dir::CopyOptions::new().content_only(true),
        ).unwrap();
        l1_vagrant_config.l2_vagrant_dir = Some(l2_vagrant_dest);
    }
    let l1_vagrant_config_file = std::fs::File::create(l1_vagrant_dest.join("config.yaml")).unwrap();
    serde_yaml::to_writer(l1_vagrant_config_file, &l1_vagrant_config).unwrap();

    println!("{:?}", project_path);
    println!("Hello, world!");
}
