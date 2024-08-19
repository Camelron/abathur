use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Name of the person to greet
    #[arg(short, long)]
    name: Option<String>,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}


fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        match args.name {
            None => println!("Hello!"),
            Some(ref name) => println!("Hello {}!", name)
        }
    }

    if let Some(command) = args.command {
        match command {
            Commands::Test { list } => {
                if list {
                    println!("Listing test values");
                } else {
                    println!("Running tests");
                }
            }
        }
    }

}
