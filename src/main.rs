use anyhow::anyhow;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::{fs, process};
use strum_macros::EnumString;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum Arch {
    #[serde(rename = "amd")]
    Amd,
    #[serde(rename = "intel")]
    Intel,
}

#[derive(Parser)]
struct CreateArgs {
    #[clap(long, help = "Path to L1 VM config yaml file")]
    l1_config: Option<PathBuf>,
    #[clap(long, help = "Path to L2 VM config yaml file")]
    l2_config: Option<PathBuf>,
    #[clap(short, long, help = "Path to project directory")]
    project_dir: Option<PathBuf>,
    #[clap(short, long, help = "Path to bench script running in L2 VM")]
    bench_script: Option<PathBuf>,
    #[clap(short, long, help = "Path to provision script for L2 VM")]
    l2_provision_script: Option<PathBuf>,
    #[clap(short, long, help = "Path to output file for benchmark results")]
    output: Option<PathBuf>,
    #[clap(
        long,
        default_value_t = false,
        help = "Path to output file for benchmark results"
    )]
    overwrite: bool,
    #[clap(long, default_value_t = false, help = "Disable nested virtualization")]
    no_nested: bool,
}

#[derive(Parser)]
struct ProvisionArgs {
    #[clap(long, help = "Path to L1 VM config yaml file")]
    l1_config: Option<PathBuf>,
    #[clap(long, help = "Path to L2 VM config yaml file")]
    l2_config: Option<PathBuf>,
    #[clap(short, long, help = "Path to project directory")]
    project_dir: Option<PathBuf>,
    #[clap(short, long, help = "Path to bench script running in L2 VM")]
    bench_script: Option<PathBuf>,
    #[clap(short, long, help = "Path to provision script for L2 VM")]
    l2_provision_script: Option<PathBuf>,
    #[clap(short, long, help = "Path to output file for benchmark results")]
    output: Option<PathBuf>,
    #[clap(
        long,
        default_value_t = false,
        help = "Overwrite existing files with original template files"
    )]
    sync: bool,
    #[clap(long, default_value_t = false, help = "Disable nested virtualization")]
    no_nested: bool,
}

#[derive(Parser)]
struct RunBenchArgs {
    #[clap(short, long, help = "Path to project directory")]
    project_dir: Option<PathBuf>,
    #[clap(short, long, help = "Path to bench script running in L2 VM")]
    bench_script: PathBuf,
    #[clap(short, long, help = "Path to output file for benchmark results")]
    output: Option<PathBuf>,
    #[clap(long, default_value_t = false, help = "Disable nested virtualization")]
    no_nested: bool,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Create vagrant directories and config files")]
    Create(CreateArgs),
    #[command(about = "Run provision script")]
    Provision(ProvisionArgs),
    #[command(about = "Run bench script")]
    RunBench(RunBenchArgs),
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
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

impl Default for CpuMode {
    fn default() -> Self {
        Self::HostModel
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct L1VagrantConfig {
    host_name: String,
    cpus: u32,
    memory: u64,
    #[serde(default)]
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
    #[serde(default)]
    cpu_mode: CpuMode,
    #[serde(default)]
    enable_network_bridge: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratedL2VagrantConfig {
    #[serde(flatten)]
    l2_vagrant_config: L2VagrantConfig,
    bench_script_path: Option<PathBuf>,
    network_interface: Option<String>,
    enable_provision_script: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct L2NoNestedVagrantConfig {
    host_name: String,
    cpus: u32,
    memory: u64,
    #[serde(default)]
    cpu_mode: CpuMode,
    network_interface: Option<String>,
    enable_provision_script: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratedL2NoNestedVagrantConfig {
    #[serde(flatten)]
    l2_vagrant_config: L2NoNestedVagrantConfig,
    bench_script_path: Option<PathBuf>,
    enable_provision_script: bool,
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
        }
    }
}

impl Default for L2NoNestedVagrantConfig {
    fn default() -> Self {
        Self {
            host_name: "l2-vagrant".to_string(),
            cpus: 2,
            memory: 2048,
            cpu_mode: CpuMode::Custom,
            network_interface: None,
            enable_provision_script: false,
        }
    }
}

fn create_l1_vagrant_directory(
    l1_vagrant_dest: &PathBuf,
    arch: Arch,
    resource_path: &PathBuf,
    l1_vagrant_config: L1VagrantConfig,
    overwrite: bool,
) -> Result<(), anyhow::Error> {
    let l1_vagrant_template_path = resource_path.join("l1-vagrant-template");

    // Create l1-vagrant directory from template if it does not exist or overwrite is true
    if l1_vagrant_dest.exists() {
        println!("l1-vagrant directory already exists");
        if overwrite {
            fs_extra::dir::remove(l1_vagrant_dest.as_path())?;
        } else {
            return Err(anyhow!("l1-vagrant directory already exists"));
        }
    }
    fs_extra::dir::create_all(l1_vagrant_dest.as_path(), false)?;
    fs_extra::dir::copy(
        l1_vagrant_template_path.as_path(),
        l1_vagrant_dest.as_path(),
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Write l1-vagrant config
    let generated_l1_config = GeneratedL1VagrantConfig {
        l1_vagrant_config,
        arch,
        l2_vagrant_dir: PathBuf::from("../l2-vagrant"),
    };
    let l1_vagrant_config_file = std::fs::File::create(l1_vagrant_dest.join("config.yaml"))?;
    serde_yaml::to_writer(l1_vagrant_config_file, &generated_l1_config)?;

    Ok(())
}

fn create_l2_no_nested_vagrant_directory(
    l2_vagrant_dest: &PathBuf,
    resource_path: &PathBuf,
    l2_vagrant_config: L2NoNestedVagrantConfig,
    bench_script_path: Option<&PathBuf>,
    l2_provision_script_path: Option<&PathBuf>,
    overwrite: bool,
) -> Result<(), anyhow::Error> {
    let l2_vagrant_template_path = resource_path.join("l2-vagrant-template");

    // Create l2-vagrant directory from template if it does not exist or overwrite is true
    if l2_vagrant_dest.exists() {
        println!("l2-vagrant directory already exists");
        if overwrite {
            fs_extra::dir::remove(l2_vagrant_dest.as_path())?;
        } else {
            return Err(anyhow!("l2-vagrant directory already exists"));
        }
    }
    fs_extra::dir::create_all(l2_vagrant_dest.as_path(), false)?;
    fs_extra::dir::copy(
        l2_vagrant_template_path.as_path(),
        l2_vagrant_dest.as_path(),
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let mut l2_vagrant_config = GeneratedL2NoNestedVagrantConfig {
        l2_vagrant_config,
        bench_script_path: None,
        enable_provision_script: false,
    };

    // Copy bench script if specified
    if let Some(bench_script_path) = &bench_script_path {
        let bench_script_dest = l2_vagrant_dest.join("run-bench.sh");
        fs_extra::file::copy(
            bench_script_path,
            &bench_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )
        .unwrap();
        l2_vagrant_config.bench_script_path = Some(fs::canonicalize(&bench_script_dest)?);
    }

    // copy provision script
    if let Some(provision_script_path) = &l2_provision_script_path {
        let provision_script_dest = l2_vagrant_dest.join("init.sh");
        fs_extra::file::copy(
            provision_script_path,
            provision_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )?;
        l2_vagrant_config.enable_provision_script = true;
    }

    // Write l2-vagrant config
    let l2_vagrant_config_file = std::fs::File::create(l2_vagrant_dest.join("config.yaml"))?;
    serde_yaml::to_writer(l2_vagrant_config_file, &l2_vagrant_config)?;

    Ok(())
}

fn create_l2_vagrant_directory(
    l2_vagrant_dest: &PathBuf,
    resource_path: &PathBuf,
    l2_vagrant_config: L2VagrantConfig,
    bench_script_path: Option<&PathBuf>,
    l2_provision_script_path: Option<&PathBuf>,
    overwrite: bool,
) -> Result<(), anyhow::Error> {
    let l2_vagrant_template_path = resource_path.join("l2-vagrant-template");

    // Create l2-vagrant directory from template if it does not exist or overwrite is true
    if l2_vagrant_dest.exists() {
        println!("l2-vagrant directory already exists");
        if overwrite {
            fs_extra::dir::remove(l2_vagrant_dest.as_path())?;
        } else {
            return Err(anyhow!("l2-vagrant directory already exists"));
        }
    }
    fs_extra::dir::create_all(l2_vagrant_dest.as_path(), false)?;
    fs_extra::dir::copy(
        l2_vagrant_template_path.as_path(),
        l2_vagrant_dest.as_path(),
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let network_interface = if l2_vagrant_config.enable_network_bridge {
        Some("eth0".to_string())
    } else {
        None
    };
    let mut l2_vagrant_config = GeneratedL2VagrantConfig {
        l2_vagrant_config,
        bench_script_path: None,
        network_interface,
        enable_provision_script: l2_provision_script_path.is_some(),
    };
    // Copy bench script if specified
    if let Some(bench_script_path) = &bench_script_path {
        let bench_script_dest = l2_vagrant_dest.join("run-bench.sh");
        fs_extra::file::copy(
            bench_script_path,
            &bench_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )
        .unwrap();
        l2_vagrant_config.bench_script_path =
            Some(PathBuf::from("/home/vagrant/l2-vagrant/run-bench.sh"));
    }

    // copy provision script
    if let Some(provision_script_path) = &l2_provision_script_path {
        let provision_script_dest = l2_vagrant_dest.join("init.sh");
        fs_extra::file::copy(
            provision_script_path,
            provision_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )?;
        l2_vagrant_config.enable_provision_script = true;
    }

    // Write l2-vagrant config
    let l2_vagrant_config_file = std::fs::File::create(l2_vagrant_dest.join("config.yaml"))?;
    serde_yaml::to_writer(l2_vagrant_config_file, &l2_vagrant_config)?;

    Ok(())
}

fn launch_vm(vagrant_dir: &PathBuf) -> Result<(), anyhow::Error> {
    // TODO: check if L1 VM already exists
    let status = process::Command::new("vagrant")
        .current_dir(vagrant_dir)
        .arg("up")
        .status()?;
    if !status.success() {
        return Err(anyhow!(format!("vagrant up failed with status: {status}")));
    }

    Ok(())
}

fn provision_vm(vagrant_dir: &PathBuf) -> Result<(), anyhow::Error> {
    let status = process::Command::new("vagrant")
        .current_dir(vagrant_dir)
        .arg("reload")
        .arg("--provision")
        .status()?;
    if !status.success() {
        return Err(anyhow!(format!(
            "vagrant reload failed with status: {status}"
        )));
    }

    Ok(())
}

fn run_l2_bench(
    l1_vagrant_dir: &PathBuf,
    output_path: Option<&PathBuf>,
) -> Result<(), anyhow::Error> {
    // run l2 bench
    let status = process::Command::new("vagrant")
        .current_dir(&l1_vagrant_dir)
        .arg("ssh")
        .arg("-c")
        .arg("./run-l2-bench.sh")
        .stdout(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(anyhow!(format!(
            "running bench script failed with status: {status}"
        )));
    }

    let output = process::Command::new("vagrant")
        .current_dir(&l1_vagrant_dir)
        .arg("ssh")
        .arg("-c")
        .arg("cat ./bench-results.txt")
        .output()?;
    if !output.status.success() {
        println!("{}", String::from_utf8(output.stderr).unwrap());
        return Err(anyhow!(format!(
            "reading bench-results.txt failed with status: {status}"
        )));
    }

    if let Some(output_path) = &output_path {
        let mut output_file = std::fs::File::create(output_path)?;
        output_file.write_all(&output.stdout)?;
        println!(
            "Bench results written to {}",
            output_path.to_str().unwrap_or("file")
        );
    }

    Ok(())
}

fn run_no_nested_l2_bench(
    l2_vagrant_dir: &PathBuf,
    output_path: Option<&PathBuf>,
) -> Result<(), anyhow::Error> {
    // run l2 bench
    // TODO: show output to console as well
    let output = process::Command::new("vagrant")
        .current_dir(&l2_vagrant_dir)
        .arg("ssh")
        .arg("-c")
        .arg("./run-bench.sh")
        .stdout(Stdio::piped())
        .output()?;
    if !output.status.success() {
        println!("{}", String::from_utf8(output.stdout).unwrap());
        return Err(anyhow!(format!(
            "running bench script failed with status: {}",
            output.status
        )));
    }

    if let Some(output_path) = &output_path {
        let mut output_file = std::fs::File::create(output_path)?;
        output_file.write_all(&output.stdout)?;
        println!(
            "Bench results written to {}",
            output_path.to_str().unwrap_or("file")
        );
    } else {
        println!("{}", String::from_utf8(output.stdout).unwrap());
    }

    Ok(())
}

fn run_create(args: CreateArgs, arch: Arch, resource_path: &PathBuf) -> Result<(), anyhow::Error> {
    let project_dir = args
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    // TODO: clean up created directories if error occurs
    if args.no_nested {
        let l2_config = if let Some(l2_config_path) = args.l2_config {
            serde_yaml::from_reader(std::fs::File::open(l2_config_path)?)?
        } else {
            L2NoNestedVagrantConfig::default()
        };
        let l2_vagrant_dest = project_dir.join("l2-vagrant-no-nested");

        create_l2_no_nested_vagrant_directory(
            &l2_vagrant_dest,
            resource_path,
            l2_config,
            args.bench_script.as_ref(),
            args.l2_provision_script.as_ref(),
            args.overwrite,
        )?;
        launch_vm(&l2_vagrant_dest)?;

        if args.bench_script.is_some() {
            run_no_nested_l2_bench(&l2_vagrant_dest, args.output.as_ref())?;
        }
    } else {
        let l1_config = if let Some(l1_config_path) = args.l1_config {
            serde_yaml::from_reader(std::fs::File::open(l1_config_path)?)?
        } else {
            L1VagrantConfig::default()
        };
        let l2_config = if let Some(l2_config_path) = args.l2_config {
            serde_yaml::from_reader(std::fs::File::open(l2_config_path)?)?
        } else {
            L2VagrantConfig::default()
        };

        let l1_vagrant_dest = project_dir.join("l1-vagrant");
        let l2_vagrant_dest = project_dir.join("l2-vagrant");
        create_l1_vagrant_directory(
            &l1_vagrant_dest,
            arch,
            resource_path,
            l1_config,
            args.overwrite,
        )?;
        create_l2_vagrant_directory(
            &l2_vagrant_dest,
            resource_path,
            l2_config,
            args.bench_script.as_ref(),
            args.l2_provision_script.as_ref(),
            args.overwrite,
        )?;

        launch_vm(&l1_vagrant_dest)?;

        if args.bench_script.is_some() {
            run_l2_bench(&l1_vagrant_dest, args.output.as_ref())?;
        }
    }

    Ok(())
}

fn update_l1_config(
    l1_vagrant_dir: &PathBuf,
    l1_config: L1VagrantConfig,
    arch: Arch,
    l2_vagrant_dir: &PathBuf,
) -> Result<(), anyhow::Error> {
    let l1_config = GeneratedL1VagrantConfig {
        l1_vagrant_config: l1_config,
        arch,
        l2_vagrant_dir: std::fs::canonicalize(&l2_vagrant_dir)?,
    };

    serde_yaml::to_writer(
        std::fs::File::create(l1_vagrant_dir.join("config.yaml"))?,
        &l1_config,
    )?;

    Ok(())
}

fn update_l2_config(
    l2_vagrant_dir: &PathBuf,
    l2_config: L2VagrantConfig,
    bench_script: Option<&PathBuf>,
    provision_script_path: Option<&PathBuf>,
) -> Result<(), anyhow::Error> {
    let network_interface = if l2_config.enable_network_bridge {
        Some("eth0".to_string())
    } else {
        None
    };
    let mut l2_config = GeneratedL2VagrantConfig {
        l2_vagrant_config: l2_config,
        bench_script_path: None,
        network_interface,
        enable_provision_script: provision_script_path.is_some(),
    };
    if let Some(bench_script_path) = bench_script {
        let bench_script_dest = l2_vagrant_dir.join("run-bench.sh");
        fs_extra::file::copy(
            bench_script_path,
            bench_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )?;
        l2_config.bench_script_path = Some(PathBuf::from("/home/vagrant/l2-vagrant/run-bench.sh"));
    }

    if let Some(provision_script_path) = provision_script_path {
        let provision_script_dest = l2_vagrant_dir.join("init.sh");
        fs_extra::file::copy(
            provision_script_path,
            provision_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )?;
    }

    serde_yaml::to_writer(
        std::fs::File::create(l2_vagrant_dir.join("config.yaml"))?,
        &l2_config,
    )?;

    Ok(())
}

fn run_provision(
    args: ProvisionArgs,
    resource_path: &PathBuf,
    arch: Arch,
) -> Result<(), anyhow::Error> {
    let project_path = args
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    let l1_vagrant_dir = project_path.join("l1-vagrant");
    let l2_vagrant_dir = project_path.join("l2-vagrant");
    let no_nested_l2_vagrant_dir = project_path.join("l2-vagrant-no-nested");
    if args.sync {
        if !args.no_nested {
            println!("copy template files to project directory");
            let l1_vagrant_template_path = resource_path.join("l1-vagrant-template");
            fs_extra::dir::copy(
                l1_vagrant_template_path.as_path(),
                l1_vagrant_dir.as_path(),
                &fs_extra::dir::CopyOptions::new()
                    .overwrite(true)
                    .content_only(true),
            )?;
            let l2_vagrant_template_path = resource_path.join("l2-vagrant-template");
            fs_extra::dir::copy(
                l2_vagrant_template_path.as_path(),
                l2_vagrant_dir.as_path(),
                &fs_extra::dir::CopyOptions::new()
                    .overwrite(true)
                    .content_only(true),
            )?;
        } else {
            let l2_vagrant_template_path = resource_path.join("l2-vagrant-template");
            fs_extra::dir::copy(
                l2_vagrant_template_path.as_path(),
                no_nested_l2_vagrant_dir.as_path(),
                &fs_extra::dir::CopyOptions::new()
                    .overwrite(true)
                    .content_only(true),
            )?;
        }
    }

    if !args.no_nested {
        if let Some(l1_config_path) = args.l1_config {
            let l1_config = serde_yaml::from_reader(std::fs::File::open(l1_config_path)?)?;
            update_l1_config(&l1_vagrant_dir, l1_config, arch, &l2_vagrant_dir)?;
        }
        if let Some(l2_config_path) = args.l2_config {
            let l2_config = serde_yaml::from_reader(std::fs::File::open(l2_config_path)?)?;
            update_l2_config(
                &l2_vagrant_dir,
                l2_config,
                args.bench_script.as_ref(),
                args.l2_provision_script.as_ref(),
            )?;
        } else if args.l2_provision_script.is_some() {
            let original_config_path = l2_vagrant_dir.join("config.yaml");
            let l2_config = serde_yaml::from_reader(std::fs::File::open(&original_config_path)?)?;
            update_l2_config(
                &l2_vagrant_dir,
                l2_config,
                args.bench_script.as_ref(),
                args.l2_provision_script.as_ref(),
            )?;
        }

        provision_vm(&l1_vagrant_dir)?;

        if args.bench_script.is_some() {
            run_l2_bench(&l1_vagrant_dir, args.output.as_ref())?;
        }
    } else {
        if let Some(l2_config_path) = args.l2_config {
            let l2_config = serde_yaml::from_reader(std::fs::File::open(l2_config_path)?)?;
            update_l2_config(
                &no_nested_l2_vagrant_dir,
                l2_config,
                args.bench_script.as_ref(),
                args.l2_provision_script.as_ref(),
            )?;
        } else if args.l2_provision_script.is_some() {
            let original_config_path = l2_vagrant_dir.join("config.yaml");
            let l2_config = serde_yaml::from_reader(std::fs::File::open(&original_config_path)?)?;
            update_l2_config(
                &l2_vagrant_dir,
                l2_config,
                args.bench_script.as_ref(),
                args.l2_provision_script.as_ref(),
            )?;
        }

        provision_vm(&no_nested_l2_vagrant_dir)?;

        if args.bench_script.is_some() {
            run_no_nested_l2_bench(&no_nested_l2_vagrant_dir, args.output.as_ref())?;
        }
    }

    Ok(())
}

// TODO: support no_nested
fn run_bench(args: RunBenchArgs) -> Result<(), anyhow::Error> {
    let project_path = args
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    if !args.no_nested {
        let l1_vagrant_dir = project_path.join("l1-vagrant");
        let l2_vagrant_dir = project_path.join("l2-vagrant");

        // Copy bench script
        let bench_script_dest = l2_vagrant_dir.join("run-bench.sh");
        fs_extra::file::copy(
            args.bench_script,
            bench_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )?;
        // Update l2-vagrant config
        let config_path = l2_vagrant_dir.join("config.yaml");
        let mut config: GeneratedL2VagrantConfig = serde_yaml::from_reader(std::fs::File::open(&config_path)?)?;
        config.bench_script_path = Some(PathBuf::from("./run-bench.sh"));
        serde_yaml::to_writer(std::fs::File::open(&config_path)?, &config)?;
        // Sync l2-vagrant directory
        process::Command::new("vagrant")
            .current_dir(&l1_vagrant_dir)
            .arg("reload")
            .status()?;

        // Boot L2 VM
        process::Command::new("vagrant")
            .current_dir(&l1_vagrant_dir)
            .arg("ssh")
            .arg("-c")
            .arg("cd /home/vagrant/l2-vagrant && vagrant up --provision")
            .status()?;

        run_l2_bench(&l1_vagrant_dir, args.output.as_ref())?;
    } else {
        let l2_vagrant_dir = project_path.join("l2-vagrant-no-nested");

        // Copy bench script
        let bench_script_dest = l2_vagrant_dir.join("run-bench.sh");
        fs_extra::file::copy(
            args.bench_script,
            bench_script_dest,
            &fs_extra::file::CopyOptions::new().overwrite(true),
        )?;
        // Update l2-vagrant config
        let config_path = l2_vagrant_dir.join("config.yaml");
        let mut config: GeneratedL2VagrantConfig = serde_yaml::from_reader(std::fs::File::open(&config_path)?)?;
        if config.bench_script_path.is_none() {
            config.bench_script_path = Some(PathBuf::from("./run-bench.sh"));
            serde_yaml::to_writer(std::fs::File::create(&config_path)?, &config)?;
        }

        // Sync l2-vagrant directory
        process::Command::new("vagrant")
            .current_dir(&l2_vagrant_dir)
            .arg("reload")
            .status()?;

        run_no_nested_l2_bench(&l2_vagrant_dir, args.output.as_ref())?;
    }
    Ok(())
}
fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let manifest_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let resource_path = manifest_path.join("resources");
    let arch = if PathBuf::from("/sys/module/kvm_intel").exists() {
        Arch::Intel
    } else if PathBuf::from("/sys/module/kvm_amd").exists() {
        Arch::Amd
    } else {
        panic!("kvm_intel or kvm_amd module is not loaded");
    };

    match args.command {
        Command::Create(args) => run_create(args, arch, &resource_path),
        Command::Provision(args) => run_provision(args, &resource_path, arch),
        Command::RunBench(args) => run_bench(args),
    }
}
