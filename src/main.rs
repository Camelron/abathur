use clap::{Arg, ArgAction, ArgMatches, Command};

fn start_vm(cmd_arguments: &ArgMatches) {
    // Start a VM
    let name = cmd_arguments.get_one::<String>("name").unwrap();
    println!("Starting VM: {}", name);
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
        _ => {
            let _ = cmd.print_help();
        }
    }
}
