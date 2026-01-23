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
                question TEXT NOT NULL,
                q_year INTEGER NOT NULL,
                q_month INTEGER NOT NULL,
                q_day INTEGER NOT NULL,
                q_hour INTEGER NOT NULL,
                q_min INTEGER NOT NULL,
                q_sec INTEGER NOT NULL
            )",
            [],
        )?;
    } else {
        // Add time columns if they don't exist (for existing databases)
        let columns_to_add = [
            ("q_year", "INTEGER DEFAULT 0"),
            ("q_month", "INTEGER DEFAULT 0"),
            ("q_day", "INTEGER DEFAULT 0"),
            ("q_hour", "INTEGER DEFAULT 0"),
            ("q_min", "INTEGER DEFAULT 0"),
            ("q_sec", "INTEGER DEFAULT 0"),
        ];

        for (col_name, col_type) in columns_to_add.iter() {
            let column_exists: bool = conn
                .prepare("PRAGMA table_info(stackoverflow_questions)")
                .and_then(|mut stmt| {
                    let mut rows = stmt.query([])?;
                    let mut found = false;
                    while let Some(row) = rows.next()? {
                        let existing_col_name: String = row.get(1)?;
                        if existing_col_name == *col_name {
                            found = true;
                            break;
                        }
                    }
                    Ok(found)
                })
                .unwrap_or(false);

            if !column_exists {
                conn.execute(
                    &format!("ALTER TABLE stackoverflow_questions ADD COLUMN {} {}", col_name, col_type),
                    [],
                )?;
            }
        }

        // Drop timestamp column if it exists
        let timestamp_exists: bool = conn
            .prepare("PRAGMA table_info(stackoverflow_questions)")
            .and_then(|mut stmt| {
                let mut rows = stmt.query([])?;
                let mut found = false;
                while let Some(row) = rows.next()? {
                    let col_name: String = row.get(1)?;
                    if col_name == "timestamp" {
                        found = true;
                        break;
                    }
                }
                Ok(found)
            })
            .unwrap_or(false);

        if timestamp_exists {
            // SQLite doesn't support DROP COLUMN directly, so we'll leave it
            // Users can manually drop it if needed
        }
    }
    Ok(())
}