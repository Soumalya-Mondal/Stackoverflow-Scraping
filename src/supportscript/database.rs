use tokio_postgres::{Client, Error, NoTls};
use std::process;

// ============================================================================
// Connect to PostgreSQL database
// ============================================================================
pub async fn connect_database(
    host: &str,
    port: &str,
    database_name: &str,
    database_user: &str,
    password: &str,
) -> Result<Client, Error> {
    // First, connect to the default 'postgres' database to check if target database exists
    let check_connection_string = format!(
        "host={} port={} dbname=postgres user={} password={}",
        host, port, database_user, password
    );

    let (check_client, check_connection) = match tokio_postgres::connect(&check_connection_string, NoTls).await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("❌ Failed to connect to PostgreSQL server: {}", e);
            process::exit(1);
        }
    };

    // Spawn check connection in background
    tokio::spawn(async move {
        if let Err(e) = check_connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    // Query to check if the database exists
    let query = format!(
        "SELECT 1 FROM pg_database WHERE datname = '{}'",
        database_name
    );

    let rows = check_client.query(&query, &[]).await?;

    if rows.is_empty() {
        eprintln!("❌ ERROR: Database '{}' does not exist in PostgreSQL server!", database_name);
        eprintln!("   Please create the database first using:");
        eprintln!("   CREATE DATABASE {};", database_name);
        process::exit(1);
    }

    println!("✅ Database '{}' found. Connecting...", database_name);

    // Now connect to the actual target database
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        host, port, database_name, database_user, password
    );

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

    // Spawn connection in background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    Ok(client)
}

// ============================================================================
// Initialize database with table creation
// ============================================================================
pub async fn init_database(client: &Client) -> Result<(), Error> {
    client.execute(
        "CREATE TABLE IF NOT EXISTS question_data (
            id SERIAL PRIMARY KEY,
            q_id BIGINT NOT NULL UNIQUE,
            q_title TEXT,
            q_year INTEGER NOT NULL,
            q_month INTEGER NOT NULL,
            q_day INTEGER NOT NULL,
            q_hours INTEGER NOT NULL,
            q_min INTEGER NOT NULL,
            q_sec INTEGER NOT NULL,
            row_inserted_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    Ok(())
}