use rusqlite::Connection;

// ============================================================================
// Check if table exists in database
// ============================================================================
pub fn table_exists(conn: &Connection, table_name: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?1"
    )?;
    let exists = stmt.exists([table_name])?;
    Ok(exists)
}

// ============================================================================
// Initialize database with table creation
// ============================================================================
pub fn init_database(conn: &Connection) -> rusqlite::Result<()> {
    if !table_exists(conn, "stackoverflow_questions")? {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stackoverflow_questions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                q_id INTEGER NOT NULL UNIQUE,
                question TEXT NOT NULL
            )",
            [],
        )?;
    }
    Ok(())
}
