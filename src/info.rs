// src/info.rs
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn print_db_info(path: &Path) -> Result<()> {
    let conn = Connection::open(path)?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )?;
    
    let table_names = stmt.query_map([], |row| row.get::<_, String>(0))?;

    println!("{:<20} | {:<10}", "Table Name", "Rows");
    println!("{}", "-".repeat(33));

    for name_result in table_names {
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

        println!("{:<20} | {:<10}", name, row_count);
    }
    Ok(())
}