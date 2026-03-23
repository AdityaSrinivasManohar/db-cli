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

    // 2. EAGERLY collect table names
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
            // SCENARIO: Table doesn't exist in target -> Create and Copy
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
            // SCENARIO: Table exists -> Check Schema
            if schemas_match(&tx, &table)? {
                let insert_verb = if no_duplicates {
                    println!("   ~ Table {:<25}: Schema matches. Duplicates will be ignored", table);
                    "INSERT OR IGNORE"
                } else {
                    println!("   ~ Table {:<25}: Schema matches. Duplicates will be added", table);
                    "INSERT"
                };

                let sql = format!(
                    "{} INTO main.\"{}\" SELECT * FROM source_db.\"{}\"",
                    insert_verb, table, table
                );

                tx.execute(&sql, [])?;
            } else {
                eprintln!(
                    "   ! Warning: Schema mismatch for '{}'. Skipping.",
                    table
                );
            }
        }
    }

    // Commit the transaction before we DETACH
    tx.commit()?;

    // 4. DETACH the source database
    target_conn.execute("DETACH DATABASE source_db", [])?;

    Ok(())
}

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
