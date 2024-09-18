pub mod daemon;

use clap::{Arg, ArgAction, ArgMatches, Command};
use reqwest::{Client};
use tokio::runtime::Runtime;
use crate::daemon::server::server_main;

const DEFAULT_PORT: &str = "4887";
const DEFAULT_ADDRESS: &str = "127.0.0.1";

async fn send_vm_to_server(vm: abathur::StartVm) {
    let client = reqwest::Client::new();
    let res = client.post(format!("http://{}:{}/start_vm", DEFAULT_ADDRESS, DEFAULT_PORT))
        .json(&vm)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.text().await.unwrap();
                println!("VM started successfully");
                println!("{}", body);
            } else {
                eprintln!("Failed to start VM: {}", response.status());
            }
        }
        Err(e) => {
            eprintln!("Error sending request: {}", e);
        }
    }
}

fn start_vm(cmd_arguments: &ArgMatches) {
    // Start a VM
    let name = cmd_arguments.get_one::<String>("name").unwrap();
    let kernel = cmd_arguments.get_one::<String>("kernel").unwrap();
    let disks: Vec<String> = cmd_arguments
        .get_many::<String>("disk")
        .unwrap_or_default()
        .map(|d| d.clone())
        .collect::<Vec<String>>();
    let cpus = cmd_arguments.get_one::<String>("cpus").unwrap();
    let memory = cmd_arguments.get_one::<String>("memory").unwrap();

    let vm = abathur::StartVm {
        name: name.to_string(),
        kernel: kernel.to_string(),
        disks: disks,
        cpus: cpus.to_string(),
        memory: memory.to_string(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        send_vm_to_server(vm).await;
    });
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
