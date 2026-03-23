mod cat;
mod info;
mod merge;

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
        path: PathBuf,
    },

    /// Print the content of a sqlite database to stdout
    Cat {
        /// Path to the sqlite database file
        path: PathBuf,
        /// Name of the table to print out
        table: String,
    },

    /// Merge multiple sqlite databases into one
    Merge {
        /// Paths to the sqlite database files to merge
        paths: Vec<PathBuf>,

        /// Output path for the merged database
        #[arg(short, long)] // This makes it -o or --output
        output: PathBuf,

        /// Prevent duplicate entries (use INSERT OR IGNORE)
        #[arg(long)]
        no_duplicates: bool,
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
            println!("Inspecting database at: {} \n", full_path.display());

            if let Err(e) = info::print_db_info(&full_path) {
                eprintln!("Error reading database: {}", e);
                exit(1);
            }
        }
        Commands::Cat { path, table } => {
            let full_path = get_absolute_path(path);
            println!(
                "Printing content of table '{}' from: {}",
                table,
                full_path.display()
            );
            if let Err(e) = cat::print_table_content(&full_path, table) {
                eprintln!("Error reading table '{}': {}", table, e);
                std::process::exit(1);
            }
        }
        Commands::Merge {
            paths,
            output,
            no_duplicates,
         } => {
            // Validate if we have at least 2 paths to merge
            if paths.len() < 2 {
                eprintln!("Error: You need at least 2 databases to perform a merge.");
                exit(1);
            }

            if let Some(parent) = output.parent() {
                if !parent.exists() && parent != std::path::Path::new("") {
                    eprintln!(
                        "Error: The parents directory for output '{}' does not exist.",
                        parent.display()
                    );
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

            // 2. Open (or create) the output database connection
            let mut target_conn = match rusqlite::Connection::open(output) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error creating output database: {}", e);
                    exit(1);
                }
            };

            // Set a busy timeout so the CLI waits 5 seconds for locks to clear instead of crashing
            let _ = target_conn.execute("PRAGMA busy_timeout = 5000;", []);

            println!("Starting merge into: {}", output.display());

            for path in &valid_paths {
                println!("-> Processing {}", path.display());

                // We pass the connection as a mutable reference so merge_databases
                // can manage its own ATTACH/DETACH and Transactions internally.
                if let Err(e) = merge::merge_databases(path, &mut target_conn, *no_duplicates) {
                    eprintln!("Error merging {}: {}", path.display(), e);
                    exit(1);
                }
            }

            println!("\nMerge completed successfully!");
        }
        Commands::Version => {
            let version = env!("CARGO_PKG_VERSION");
            println!("db-cli {}", version);
        }
    }
}
