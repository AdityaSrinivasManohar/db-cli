// src/info.rs
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn print_db_info(path: &Path) -> Result<()> {
    let conn = Connection::open(path)?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )?;
    let table_names = stmt.query_map([], |row| row.get::<_, String>(0))?;

    println!("{:<3} | {:<20} | {:<10} | {:<10}", "No.", "Table Name", "Rows", "Columns");
    println!("{}", "-".repeat(50));

    for (index, name_result) in table_names.enumerate() {
        let name = name_result?;
        let row_count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM \"{}\"", name),
            [],
            |row| row.get(0), // a closure in rust is similar to lambda in python,
                                               // it allows us to define an inline function that can 
                                               // capture variables from its surrounding scope. 
                                               // In this case, we are using it to extract the row 
                                               // count from the result of the query.
        )?;
        
        // 2. Get Column Count
        // PRAGMA table_info returns one row for every column in the table
        let mut col_stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", name))?;
        let column_results = col_stmt.query_map([], |_| Ok(()))?; 
        let column_count = column_results.count();

        println!("{:<3} | {:<20} | {:<10} | {:<10}", index + 1, name, row_count, column_count);
    }
    Ok(())
}