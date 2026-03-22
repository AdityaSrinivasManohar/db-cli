use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::exit;

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
        path: PathBuf
    },

    /// Print the content of a sqlite database to stdout
    Cat{
        /// Path to the sqlite database file
        path: PathBuf
    },

    /// Merge multiple sqlite databases into one
    Merge{
        /// Paths to the sqlite database files to merge
        paths: Vec<PathBuf>,

        /// Output path for the merged database
        #[arg(short, long)] // This makes it -o or --output
        output: PathBuf,
    },

    /// Version
    Version,
}

fn get_absolute_path(path: &PathBuf) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {} - {}", path.display(), e);
            exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info { path } => {
            let full_path = get_absolute_path(path);
            println!("Inspecting absolute path: {}", full_path.display());
        }
        Commands::Cat { path } => {
            let full_path = get_absolute_path(path);
            println!("Printing content of: {}", full_path.display());
        }
        Commands::Merge { paths, output } => {
            // Validate if we have at least 2 paths to merge
            if paths.len() < 2 {
                eprintln!("Error: You need at least 2 databases to perform a merge.");
                exit(1);
            }

            if let Some(parent) = output.parent() {
                if !parent.exists() && parent != std::path::Path::new("") {
                    eprintln!("Error: The parents directory for output '{}' does not exist.", parent.display());
                    exit(1);
                }
            }

            let mut valid_paths = Vec::new();
            for path in paths {
                let full_path = get_absolute_path(path);
                valid_paths.push(full_path);
            }

            // If we got here, all paths are valid
            println!("Merging files:");
            for path in &valid_paths {
                println!("- {}", path.display());
            }
            println!("into {}", output.display());
        }
        Commands::Version => {
            let version = env!("CARGO_PKG_VERSION");
            println!("db-cli {}", version);
        }
    }
}