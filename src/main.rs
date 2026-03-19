use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// A cli tool for interacting with sqlite databases
#[derive(Subcommand)]
enum Commands {
    /// Util to provide info on a given sqlite database
    Info { 
        /// Path to the sqlite database file
        path: String
    },

    /// Print the content of a sqlite database to stdout
    Cat{
        /// Path to the sqlite database file
        path: String
    },

    /// Merge multiple sqlite databases into one
    Merge{
        /// Paths to the sqlite database files to merge
        paths: Vec<String>,

        /// Output path for the merged database
        #[arg(short, long)] // This makes it -o or --output
        output: String,
    },

    /// Version
    Version,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info { path } => {
            println!("You want to inspect: {}", path);
        }
        Commands::Cat { path } => {
            println!("You want to print the content of: {}", path);
        }
        Commands::Merge { paths, output } => {
            println!("Merging databases: {:?} into {}", paths, output);
        }
        Commands::Version => {
            let version = env!("CARGO_PKG_VERSION");
            println!("db-cli {}", version);
        }
    }
}