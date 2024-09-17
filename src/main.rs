pub mod daemon;

use clap::{Arg, ArgAction, ArgMatches, Command};
use tokio::runtime::Runtime;
use crate::daemon::server::server_main;


fn start_vm(cmd_arguments: &ArgMatches) {
    // Start a VM
    // Can simply unwrap here because we know the arguments are required
    let name = cmd_arguments.get_one::<String>("name").unwrap();
    let kernel = cmd_arguments.get_one::<String>("kernel").unwrap();
    let disks: Vec<&str> = cmd_arguments
        .get_many::<String>("disk")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let cpus = cmd_arguments.get_one::<String>("cpus").unwrap();
    let memory = cmd_arguments.get_one::<String>("memory").unwrap();

    println!("Starting VM: {}", name);
    println!("Kernel: {}", kernel);
    println!("Disks: {:?}", disks);
    println!("CPUs: {}", cpus);
    println!("Memory: {}", memory);

    let clh_command = std::process::Command::new("cloud-hypervisor")
        .env("PATH", "/bin")
        // .arg("-v")
        .arg("--kernel")
        .arg(kernel)
        .arg("--disk")
        .args(disks.iter().map(|d| format!("path={}", d)))
        .arg("--cpus")
        .arg(format!("boot={}", cpus))
        .arg("--memory")
        .arg(format!("size={}", memory))
        .spawn();

    let mut clh_process = match clh_command {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start cloud-hypervisor: {}", e);
            return;
        }
    };

    let _ = clh_process.wait();
}

fn create_app() -> Command {
    let app = Command::new("abathur")
        .version("0.1.0")
        .about("Abathur: a simple VM orchestrator")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Start a VM")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("Name of the VM to start")
                        .required(true),
                )
                .arg(
                    Arg::new("kernel")
                        .short('k')
                        .long("kernel")
                        .help("Kernel (or firmware) image to boot the VM")
                        .required(true),
                )
                .arg(
                    Arg::new("disk")
                        .short('d')
                        .long("disk")
                        .help("disk image to add to the VM")
                        .action(ArgAction::Append)
                        .required(true),
                )
                .arg(
                    Arg::new("cpus")
                        .long("cpus")
                        .help("number of cpus to allocate to the VM")
                        .default_value("2")
                )
                .arg(
                    Arg::new("memory")
                        .long("memory")
                        .help("amount of memory to allocate to the VM")
                        .default_value("2048M")
                )
        )
        .subcommand(
            Command::new("daemon")
                .about("The Abathur daemon controlling persistent state and VM lifecycle")
                .subcommand(
                    Command::new("start")
                        .about("Start the Abathur daemon")
                        .arg(
                            Arg::new("port")
                                .short('p')
                                .long("port")
                                .help("Port to listen on")
                                .default_value("4887")
                        )
                )
        );

    app
}

fn main() {
    let mut cmd = create_app();
    let args = cmd.clone().get_matches();

    match args.subcommand() {
        Some(("start", start_args)) => {
            start_vm(start_args);
        }
        Some(("daemon", daemon_args)) => {
            match daemon_args.subcommand() {
                Some(("start", start_args)) => {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        server_main(start_args).await;
                    });
                }
                _ => {
                    let _ = cmd.print_help();
                }
            }
        }
        _ => {
            let _ = cmd.print_help();
        }
    }
}
