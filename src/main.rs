use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum Arch {
    #[serde(rename = "amd")]
    Amd,
    #[serde(rename = "intel")]
    Intel,
}

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
    kvm_options: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratedL1VagrantConfig {
    #[serde(flatten)]
    l1_vagrant_config: L1VagrantConfig,

    arch: Arch,
    l2_vagrant_dir: PathBuf,
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
            kvm_options: HashMap::new(),
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

struct CreateVagrantDirectoriesResult {
    l1_vagrant_dir: PathBuf,
    l2_vagrant_dir: PathBuf,
    l1_exists: bool,
    l2_exists: bool,
    l1_config_updated: bool,
    l2_config_updated: bool,
}

fn create_vagrant_directories(
    args: &Args,
    project_path: &PathBuf,
    arch: Arch,
) -> CreateVagrantDirectoriesResult {
    let l1_vagrant_template_path = project_path.join("resources").join("l1-vagrant-template");
    let dest_base_dir = if let Some(dest) = &args.dest {
        dest.clone()
    } else {
        std::env::current_dir().unwrap()
    };
    let l1_vagrant_dest = dest_base_dir.join("l1-vagrant");
    // read L1 config file
    let mut l1_vagrant_config = if let Some(config_path) = &args.l1_config {
        let config_file = std::fs::File::open(config_path).unwrap();
        serde_yaml::from_reader(config_file).unwrap()
    } else {
        L1VagrantConfig::default()
    };
    // read L2 config file
    let mut l2_vagrant_config = if let Some(config_path) = &args.l2_config {
        let config_file = std::fs::File::open(config_path).unwrap();
        serde_yaml::from_reader(config_file).unwrap()
    } else {
        L2VagrantConfig::default()
    };

    // Create l1-vagrant directory from template if it does not exist
    let l1_exists = if l1_vagrant_dest.exists() {
        println!("l1-vagrant directory already exists");
        true
    } else {
        fs_extra::dir::create_all(l1_vagrant_dest.as_path(), false).unwrap();
        fs_extra::dir::copy(
            l1_vagrant_template_path.as_path(),
            l1_vagrant_dest.as_path(),
            &fs_extra::dir::CopyOptions::new().content_only(true),
        )
        .unwrap();
        false
    };

    // Create l2-vagrant directory from template if it does not exist
    let l2_vagrant_dest = dest_base_dir.join("l2-vagrant");
    let l2_vagrant_template_path = project_path.join("resources").join("l2-vagrant-template");
    let l2_exists = if l2_vagrant_dest.exists() {
        println!("l2-vagrant directory already exists");
        if args.l2_config.is_some() {
            println!("l2_config will be ignored");
        }
        true
    } else {
        fs_extra::dir::create_all(l2_vagrant_dest.as_path(), false).unwrap();
        fs_extra::dir::copy(
            l2_vagrant_template_path.as_path(),
            l2_vagrant_dest.as_path(),
            &fs_extra::dir::CopyOptions::new().content_only(true),
        )
        .unwrap();
        false
    };

    let mut l1_config_updated = false;
    // Write l1-vagrant config if l1-vagrant directory is created or l1_config is specified by args
    if !l1_exists || args.l1_config.is_some() {
        let generated_l1_config = GeneratedL1VagrantConfig {
            l1_vagrant_config,
            arch,
            l2_vagrant_dir: std::fs::canonicalize(&l2_vagrant_dest).unwrap(),
        };
        let l1_vagrant_config_file =
            std::fs::File::create(l1_vagrant_dest.join("config.yaml")).unwrap();
        serde_yaml::to_writer(l1_vagrant_config_file, &generated_l1_config).unwrap();
        l1_config_updated = true;
    }

    // Write bench script
    if let Some(bench_script_path) = &args.bench_script {
        let bench_script_dest = l2_vagrant_dest.join("run-bench.sh");
        fs_extra::file::copy(
            bench_script_path,
            bench_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )
        .unwrap();
        l2_vagrant_config.bench_script_path =
            Some(PathBuf::from("/home/vagrant/l2-vagrant/run-bench.sh"));
    }

    let mut l2_config_updated = false;
    let l2_vagrant_config_path = l2_vagrant_dest.join("config.yaml");
    // Write l2-vagrant config if l2-vagrant directory is created or l2_config is specified by args
    if !l2_exists || args.l2_config.is_some() {
        let l2_vagrant_config_file = std::fs::File::create(&l2_vagrant_config_path).unwrap();
        serde_yaml::to_writer(l2_vagrant_config_file, &l2_vagrant_config).unwrap();
        l2_config_updated = true;
    }
    if l2_exists && args.l2_config.is_none() && args.bench_script.is_some() {
        l2_vagrant_config =
            serde_yaml::from_reader(std::fs::File::open(&l2_vagrant_config_path).unwrap()).unwrap();
        l2_vagrant_config.bench_script_path =
            Some(PathBuf::from("/home/vagrant/l2-vagrant/run-bench.sh"));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&l2_vagrant_config_path)
            .unwrap();
        serde_yaml::to_writer(file, &l2_vagrant_config).unwrap();
        l2_config_updated = true;
    }

    CreateVagrantDirectoriesResult {
        l1_vagrant_dir: l1_vagrant_dest,
        l2_vagrant_dir: l2_vagrant_dest,
        l1_exists,
        l2_exists,
        l1_config_updated,
        l2_config_updated,
    }
}

fn launch_l1_vm(_args: &Args, create_result: &CreateVagrantDirectoriesResult) {
    let l1_vagrant_dir = &create_result.l1_vagrant_dir;
    // vagrant up
    if !create_result.l1_exists {
        let mut child = Command::new("vagrant")
            .current_dir(l1_vagrant_dir)
            .arg("up")
            .spawn()
            .unwrap();
        child.wait().unwrap();
    } else if create_result.l1_config_updated {
        // TODO: `vagrat up` if L1 VM is not created
        let mut child = Command::new("vagrant")
            .current_dir(l1_vagrant_dir)
            .arg("reload")
            .arg("--provision")
            .spawn()
            .unwrap();
        child.wait().unwrap();
    }
    // TODO: re-provision L2 VM if L2 VM config is updated
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
    let arch = if PathBuf::from("/sys/module/kvm_intel").exists() {
        Arch::Intel
    } else if PathBuf::from("/sys/module/kvm_amd").exists() {
        Arch::Amd
    } else {
        panic!("kvm_intel or kvm_amd module is not loaded");
    };

    let result = create_vagrant_directories(&args, &project_path, arch);

    launch_l1_vm(&args, &result);
    if args.bench_script.is_none() {
        println!("No bench script specified, skipping bench");
    } else {
        run_l2_bench(&args);
    }
}
