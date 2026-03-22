// src/cat.rs
use rusqlite::{Connection, Result, types::ValueRef};
use std::path::Path;

pub fn print_table_content(path: &Path, table_name: &str) -> Result<()> {
    let conn = Connection::open(path)?;

    // 1. Prepare the query (using double quotes for the table name)
    let mut stmt = conn.prepare(&format!("SELECT * FROM \"{}\"", table_name))?;
    
    // 2. Get column names for the header
    let col_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    // Print Header
    println!("{}", col_names.join(" | "));
    println!("{}", "-".repeat(col_names.len() * 15));

    // 3. Query the rows
    let mut rows = stmt.query([])?;

    // 4. Iterate through each row
    while let Some(row) = rows.next()? {
        let mut row_values = Vec::new();

        // Iterate through each column in the row
        for i in 0..col_names.len() {
            // Get a "ValueRef" which can be any SQLite type
            let value = row.get_ref(i)?;
            
            // Convert the SQLite value to a printable string
            let val_str = match value {
                ValueRef::Null => "NULL".to_string(),
                ValueRef::Integer(i) => i.to_string(),
                ValueRef::Real(f) => f.to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                ValueRef::Blob(b) => format!("<{} bytes>", b.len()),
            };
            row_values.push(val_str);
        }
        println!("{}", row_values.join(" | "));
    }

    Ok(())
}