// src/diff.rs
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn compare_tables(
    db_a_path: &Path,
    db_b_path: &Path,
    table: &str,
) -> Result<()> {
    let conn = Connection::open(db_a_path)?;

    // Attach DB B
    let attach_query = format!("ATTACH DATABASE '{}' AS db_b", db_b_path.display());
    conn.execute(&attach_query, [])?;

    println!("Comparing table '{}'...", table);
    println!("{}", "-".repeat(40));

    // 1. Find rows in A that are NOT in B
    let _ = {
        let sql = format!(
            "SELECT * FROM main.\"{}\" EXCEPT SELECT * FROM db_b.\"{}\"",
            table, table
        );
        let mut stmt = conn.prepare(&sql)?;
        let column_count = stmt.column_count();

        println!("Rows unique to {}:", db_a_path.file_name().unwrap().to_string_lossy());
        let mut rows = stmt.query([])?;
        render_diff_rows(&mut rows, column_count)?;
    };

    println!("\n{}", "-".repeat(40));

    // 2. Find rows in B that are NOT in A
    let _ = {
        let sql = format!(
            "SELECT * FROM db_b.\"{}\" EXCEPT SELECT * FROM main.\"{}\"",
            table, table
        );
        let mut stmt = conn.prepare(&sql)?;
        let column_count = stmt.column_count();

        println!("Rows unique to {}:", db_b_path.file_name().unwrap().to_string_lossy());
        let mut rows = stmt.query([])?;
        render_diff_rows(&mut rows, column_count)?;
    };

    Ok(())
}

fn render_diff_rows(rows: &mut rusqlite::Rows, col_count: usize) -> Result<()> {
    let mut found = false;
    while let Some(row) = rows.next()? {
        found = true;
        let mut values = Vec::new();
        for i in 0..col_count {
            let value = row.get::<_, rusqlite::types::Value>(i)?;
            // Explicitly match the type and format it to a String
            let val_str = match value {
                rusqlite::types::Value::Null => "NULL".to_string(),
                rusqlite::types::Value::Integer(i) => i.to_string(),
                rusqlite::types::Value::Real(f) => f.to_string(),
                rusqlite::types::Value::Text(t) => t,
                rusqlite::types::Value::Blob(b) => format!("<Blob {}b>", b.len()),
            };
            values.push(val_str);
        }
        println!("  | {}", values.join(" | "));
    }

    if !found {
        println!("  (No differences found)");
    }
    Ok(())
}
