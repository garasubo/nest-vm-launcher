use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use clap::Parser;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Parser)]
struct Args {
    #[clap(long, help = "Path to L1 VM config yaml file")]
    l1_config: Option<PathBuf>,
    #[clap(long, help = "Path to L2 VM config yaml file")]
    l2_config: Option<PathBuf>,
    #[clap(short, long, help = "Path to bench script running in L2 VM")]
    bench_script: Option<PathBuf>,
    #[clap(short, long, help = "Path to project directory")]
    dest: Option<PathBuf>,
    #[clap(short, long, help = "Path to output file for benchmark results")]
    output: Option<PathBuf>,
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
    bench_script_path: Option<PathBuf>,
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
            bench_script_path: None,
        }
    }
}

fn create_vagrant_directories(args: &Args, project_path: &PathBuf) {
    let l1_vagrant_template_path = project_path.join("resources").join("l1-vagrant-template");
    let dest_base_dir = if let Some(dest) = &args.dest {
        dest.clone()
    } else {
        std::env::current_dir().unwrap()
    };
    let l1_vagrant_dest = dest_base_dir.join("l1-vagrant");
    println!("{:?}", l1_vagrant_template_path);
    println!("{:?}", l1_vagrant_template_path);
    let mut l1_vagrant_config = if let Some(config_path) = &args.l1_config {
        let config_file = std::fs::File::open(config_path).unwrap();
        serde_yaml::from_reader(config_file).unwrap()
    } else {
        L1VagrantConfig::default()
    };
    let mut l2_vagrant_config = if let Some(config_path) = &args.l2_config {
        let config_file = std::fs::File::open(config_path).unwrap();
        serde_yaml::from_reader(config_file).unwrap()
    } else {
        L2VagrantConfig::default()
    };
    fs_extra::dir::create_all(l1_vagrant_dest.as_path(), false).unwrap();
    fs_extra::dir::copy(
        l1_vagrant_template_path.as_path(),
        l1_vagrant_dest.as_path(),
        &fs_extra::dir::CopyOptions::new().content_only(true),
    ).unwrap();
    let l2_vagrant_dest = if l1_vagrant_config.l2_vagrant_dir.is_none() {
        let l2_vagrant_dest = dest_base_dir.join("l2-vagrant");
        let l2_vagrant_template_path = project_path.join("resources").join("l2-vagrant-template");
        fs_extra::dir::create_all(l2_vagrant_dest.as_path(), false).unwrap();
        fs_extra::dir::copy(
            l2_vagrant_template_path.as_path(),
            l2_vagrant_dest.as_path(),
            &fs_extra::dir::CopyOptions::new().content_only(true),
        ).unwrap();
        l1_vagrant_config.l2_vagrant_dir = Some(std::fs::canonicalize(&l2_vagrant_dest).unwrap());
        l2_vagrant_dest
    } else {
        l1_vagrant_config.l2_vagrant_dir.clone().unwrap()
    };
    let l1_vagrant_config_file = std::fs::File::create(l1_vagrant_dest.join("config.yaml")).unwrap();
    serde_yaml::to_writer(l1_vagrant_config_file, &l1_vagrant_config).unwrap();
    if let Some(bench_script_path) = &args.bench_script {
        let bench_script_dest = l2_vagrant_dest.join("run-bench.sh");
        fs_extra::file::copy(bench_script_path, bench_script_dest, &fs_extra::file::CopyOptions::new()).unwrap();
        l2_vagrant_config.bench_script_path = Some(PathBuf::from("/home/vagrant/l2-vagrant/run-bench.sh"));
    }
    let l2_vagrant_config_file = std::fs::File::create(l2_vagrant_dest.join("config.yaml")).unwrap();
    serde_yaml::to_writer(l2_vagrant_config_file, &l2_vagrant_config).unwrap();
}

fn launch_l1_vm(args: &Args) {
    let dest_base_dir = if let Some(dest) = &args.dest {
        dest.clone()
    } else {
        std::env::current_dir().unwrap()
    };
    let l1_vagrant_dir = dest_base_dir.join("l1-vagrant");
    // vagrant up
    let mut child = Command::new("vagrant")
        .current_dir(l1_vagrant_dir)
        .arg("up")
        .spawn()
        .unwrap();
    child.wait().unwrap();
}

fn run_l2_bench(args: &Args) {
    let dest_base_dir = if let Some(dest) = &args.dest {
        dest.clone()
    } else {
        std::env::current_dir().unwrap()
    };
    let l1_vagrant_dir = dest_base_dir.join("l1-vagrant");
    // run l2 bench
    let mut child = Command::new("vagrant")
        .current_dir(&l1_vagrant_dir)
        .arg("ssh")
        .arg("-c")
        .arg("./run-l2-bench.sh")
        .spawn()
        .unwrap();
    child.wait().unwrap();

    let output = Command::new("vagrant")
        .current_dir(&l1_vagrant_dir)
        .arg("ssh")
        .arg("-c")
        .arg("cat ./bench-results.txt")
        .output()
        .unwrap();
    if let Some(output_path) = &args.output {
        let mut output_file = std::fs::File::create(output_path).unwrap();
        output_file.write_all(&output.stdout).unwrap();
    } else {
        println!("{}", String::from_utf8(output.stdout).unwrap());
    }
}

fn main() {
    let args = Args::parse();
    let project_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    create_vagrant_directories(&args, &project_path);
    launch_l1_vm(&args);
    if args.bench_script.is_none() {
        println!("No bench script specified, skipping bench");
    } else {
        run_l2_bench(&args);
    }
}
