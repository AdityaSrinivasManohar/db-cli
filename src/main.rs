use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info { path } => {
            match path.canonicalize() {
                Ok(full_path) => println!("Inspecting absolute path: {}", full_path.display()),
                Err(err) => {
                    eprintln!("Could not find file at {} \nError: {}", path.display(), err);
                    return;
                }
            }
        }
        Commands::Cat { path } => {
            match path.canonicalize() {
                Ok(full_path) => println!("Printing content of: {}", full_path.display()),
                Err(err) => {
                    eprintln!("Could not find file at {} \nError: {}", path.display(), err);
                    return;
                }
            }
        }
        Commands::Merge { paths, output } => {
            // Validate if we have at least 2 paths to merge
            if paths.len() < 2 {
                eprintln!("Error: You need at least 2 databases to perform a merge.");
                return;
            }

            let mut valid_paths = Vec::new();
            for path in paths {
                match path.canonicalize() {
                    Ok(full_path) => {valid_paths.push(full_path);},
                    Err(err) => {
                        eprintln!("Could not find file at {} \nError: {}", path.display(), err);
                        return;
                    },
                }
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