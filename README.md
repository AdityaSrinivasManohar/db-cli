# db-cli: A High-Performance SQLite Swiss Army Knife

db-cli is a command-line utility built in Rust for inspecting, querying, and intelligently merging SQLite databases. Designed for developers and engineers, it handles schema validation, dynamic data typing, and complex merges where primary key conflicts are a concern.

## Key Features
- Intelligent Merging: Merges multiple databases into one using a "Strict Schema, Flexible Data" policy.

- Auto-ID Resolution: Automatically detects Primary Keys and strips them during merges to prevent UNIQUE constraint failures, allowing for seamless data appending.

- Dynamic Inspection: View table schemas, row counts, and column counts at a glance.

- Schema Safety: Prevents merging when table structures mismatch, ensuring data integrity.

- Atomic Operations: Uses SQL Transactions to ensure that if a merge fails halfway through a file, the output database remains untouched.

## Installation
Ensure you have the Rust toolchain installed.


```
# install using cargo
cargo install db-cli

# build from source
git clone https://github.com/AdityaSrinivasManohar/db-cli.git
cd db-cli
cargo build --release
```
The binary will be available at ./target/release/db-cli.

### 📖 Usage Guide
#### 1. Inspecting a Database (info)
Provides a high-level summary of the database structure, including the number of tables, rows per table, and column counts.

```
db-cli info <path_to_sqlite_file>
```
#### 2. Printing Table Content (cat)
Displays the content of a specific table. It dynamically handles SQLite types (Integer, Text, Blob, Null) and formats them for the terminal.

```
db-cli cat <path_to_sqlite_file> <table_name>
```
#### 3. Merging Databases (merge)
Combines multiple source databases into a single output file.

**Standard Merge (Appends with Auto-ID)**:
If a table exists in both files, db-cli will strip the Primary Key and let the target database assign new IDs, preventing crashes.

```
db-cli merge file1.sqlite file2.sqlite --output merged.sqlite
```

**Merge with De-duplication**:
Uses INSERT OR IGNORE logic. If a record with the same Primary Key already exists, the new record is skipped.

```
db-cli merge file1.sqlite file2.sqlite --output merged.sqlite --no-duplicates
```
