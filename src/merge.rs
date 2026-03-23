use rusqlite::{Connection, Result};
use std::path::Path;

pub fn merge_databases(
    source_path: &Path,
    target_conn: &mut Connection,
    no_duplicates: bool,
) -> Result<()> {
    // 1. ATTACH the source database
    // We do this BEFORE starting a transaction.
    let attach_query = format!("ATTACH DATABASE '{}' AS source_db", source_path.display());
    target_conn.execute(&attach_query, [])?;

    // 2. get table names
    // We pull these into a Vec so we aren't holding a cursor open while we write.
    let tables: Vec<String> = {
        let mut stmt = target_conn.prepare(
            "SELECT name FROM source_db.sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut names = Vec::new();
        for name in rows {
            names.push(name?);
        }
        names
    };

    // 3. Start a Transaction for the actual data movement
    // This ensures that if one table fails, the whole file merge is rolled back.
    let tx = target_conn.transaction()?;

    for table in tables {
        // Check if table exists in target (main)
        let exists: bool = tx.query_row(
            "SELECT count(*) FROM main.sqlite_master WHERE type='table' AND name = ?",
            [&table],
            |row| row.get::<_, i64>(0).map(|count| count > 0),
        )?;

        if !exists {
            // Table doesn't exist in target -> Create and Copy
            println!("   + New table: {:<25}: Creating and copying...", table);

            let create_sql: String = tx.query_row(
                "SELECT sql FROM source_db.sqlite_master WHERE name = ?",
                [&table],
                |row| row.get(0),
            )?;

            tx.execute(&create_sql, [])?;

            let move_sql = format!(
                "INSERT INTO main.\"{}\" SELECT * FROM source_db.\"{}\"",
                table, table
            );
            tx.execute(&move_sql, [])?;
        } else {
            // Table exists -> Check Schema
            if schemas_match(&tx, &table)? {
                let sql = if no_duplicates {
                    println!(
                        "   ~ Table {:<25}: Skipping duplicates via INSERT OR IGNORE",
                        table
                    );
                    format!(
                        "INSERT OR IGNORE INTO main.\"{}\" SELECT * FROM source_db.\"{}\"",
                        table, table
                    )
                } else {
                    println!(
                        "   ~ Table {:<25}: Appending with Auto-ID (stripping PK)",
                        table
                    );

                    // 1. Get column names, filtering out the Primary Key (pk)
                    let mut col_stmt = tx.prepare(&format!("PRAGMA table_info(\"{}\")", table))?;
                    let columns: Vec<String> = col_stmt
                        .query_map([], |row| {
                            let name: String = row.get(1)?;
                            let is_pk: i32 = row.get(5)?; // pk column is index 5
                            Ok((name, is_pk))
                        })?
                        .filter_map(|res| {
                            let (name, is_pk) = res.ok()?;
                            if is_pk == 0 {
                                Some(format!("\"{}\"", name))
                            } else {
                                None
                            }
                        })
                        .collect();

                    let col_list = columns.join(", ");

                    // 2. Build the specific INSERT that lets the target handle the ID
                    format!(
                        "INSERT INTO main.\"{}\" ({}) SELECT {} FROM source_db.\"{}\"",
                        table, col_list, col_list, table
                    )
                };

                tx.execute(&sql, [])?;
            } else {
                eprintln!("   ! Warning: Schema mismatch for '{}'. Skipping.", table);
            }
        }
    }

    // Commit the transaction before we DETACH
    tx.commit()?;

    // 4. DETACH the source database
    target_conn.execute("DETACH DATABASE source_db", [])?;

    Ok(())
}

// Function to check if the schema of a table in the target database matches the source database.
fn schemas_match(conn: &Connection, table_name: &str) -> Result<bool> {
    // Compare the CREATE TABLE statements for equality
    let sql_main: String = conn.query_row(
        "SELECT sql FROM main.sqlite_master WHERE name = ?",
        [table_name],
        |row| row.get(0),
    )?;
    let sql_source: String = conn.query_row(
        "SELECT sql FROM source_db.sqlite_master WHERE name = ?",
        [table_name],
        |row| row.get(0),
    )?;

    Ok(sql_main == sql_source)
}
