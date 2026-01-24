# Stack Overflow Scraper

A simple and efficient asynchronous web scraper built in Rust to fetch all question titles and their corresponding IDs from Stack Overflow. The scraped data is stored in a PostgreSQL database.

## Features

-   **Asynchronous Fetching**: Uses `tokio` and `reqwest` for non-blocking HTTP requests, allowing for efficient scraping.
-   **Modular Architecture**: Separates concerns into dedicated modules (`database.rs`, `fileops.rs`, `questionsvalue.rs`) for better code organization and maintainability.
-   **Polite Scraping**: Implements random delays (0.1-1.9 seconds) between requests to avoid overwhelming the server and respect rate limits.
-   **Resilient**: Handles network errors and non-successful HTTP responses gracefully, logging failed pages for manual review.
-   **PostgreSQL Database**: Stores scraped data in a PostgreSQL database with a unique constraint on question IDs and automatic timestamp tracking.
-   **Timestamp Extraction**: Parses the question's creation date and time, plus tracks insertion time automatically.
-   **Duplicate Prevention**: The table has a UNIQUE constraint on `q_id`, and the INSERT statement uses `ON CONFLICT (q_id) DO NOTHING` to silently ignore duplicate question IDs.
-   **Continuous Scraping**:
    -   Processes all remaining pages from the last checkpoint down to page 1.
    -   Automatically resumes from where it left off on subsequent runs using `output/LastPage.txt`.
    -   Scrapes in reverse chronological order (newest questions first).
-   **Error Logging**: Failed page fetches are logged to `output/LostPage.txt` for later retry or investigation.

## Project Structure

```
Stackoverflow-Scraper/
├── src/
│   ├── main.rs                  # Main entry point and orchestration
│   └── supportscript/
│       ├── mod.rs               # Module declaration
│       ├── database.rs          # PostgreSQL connection and operations
│       ├── fileops.rs           # File I/O operations
│       └── questionsvalue.rs    # HTML parsing for question data
├── output/                      # Created automatically
│   ├── LastPage.txt             # Tracks the last processed page number
│   └── LostPage.txt             # Logs failed page fetches
├── Cargo.toml                  # Project dependencies and configuration
└── README.md                   # This file
```

## How It Works

1.  **Initialization**: The scraper creates an `output` directory, connects to a PostgreSQL database using hardcoded credentials, and creates/updates the `question_data` table schema.

2.  **Metadata Fetch**: It makes an initial request to Stack Overflow to determine the total number of questions and calculates the total number of pages to scrape (50 questions per page).

3.  **Resume from Last Session**: It reads `output/LastPage.txt` to find the last processed page and resumes scraping from there. If the file doesn't exist or is empty, it starts from the last page (newest questions).

4.  **Continuous Scraping Loop**: It processes all pages from the starting point down to page 1 in reverse chronological order:
    -   For each page, it sends an HTTP GET request with a `User-Agent` header.
    -   Adds a random delay (0.1-1.9 seconds) between requests to be a polite scraper.
    -   Uses the `questionsvalue` module to parse the HTML response and extract question titles, IDs, and timestamps.
    -   Checks for duplicate question IDs in the database before insertion using functions from the `database` module.

5.  **Data Storage**: For each question, it inserts the new question data (ID, title as `titel`, and timestamp components) into the `question_data` table. If a question ID already exists, the `ON CONFLICT (q_id) DO NOTHING` clause silently ignores the duplicate. The `row_inserted_at` column is automatically populated with the current timestamp. After successfully processing a page, it updates `output/LastPage.txt`.

6.  **Error Handling**: Failed page fetches are logged to `output/LostPage.txt` with their page numbers for manual investigation or retry.

7.  **Completion**: The process completes after processing all remaining pages. Run the scraper again to continue if interrupted.

## Dependencies

The project relies on the following Rust crates:

-   `tokio` (v1.49.0): Asynchronous runtime.
-   `reqwest` (v0.13.1): HTTP client.
-   `scraper` (v0.25.0): HTML parsing and CSS selector matching.
-   `tokio-postgres` (v0.7): PostgreSQL database client.
-   `rand` (v0.9.2): Random number generation.
-   `chrono` (v0.4.43): Date and time library.

## Database Schema

```sql
CREATE TABLE question_data (
    id SERIAL PRIMARY KEY,
    q_id BIGINT NOT NULL UNIQUE,
    titel TEXT,
    q_year INTEGER NOT NULL,
    q_month INTEGER NOT NULL,
    q_day INTEGER NOT NULL,
    q_hours INTEGER NOT NULL,
    q_min INTEGER NOT NULL,
    q_sec INTEGER NOT NULL,
    row_inserted_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Database Configuration

The database connection parameters are hardcoded in `src/main.rs`:

- **Host**: localhost
- **Port**: 5432
- **Database Name**: stackoverflow_data
- **Username**: soumalya
- **Password**: Soumalya@1996

**Prerequisites**: Ensure you have PostgreSQL installed and running with the database `stackoverflow_data` created before running the scraper.

## Configuration

You can modify the following constants in `src/main.rs`:

-   `USER_AGENT` - Browser user agent string sent with requests.
-   Database connection parameters (host, port, database_name, database_user, password) - hardcoded in the main function.

## How to Run

1.  **Prerequisites:**
    - Install and configure PostgreSQL
    - Create a database named `stackoverflow_data`
    - Ensure the user `soumalya` has access with password `Soumalya@1996`

2.  **Clone the repository:**
    ```sh
    git clone <repository-url>
    cd Stackoverflow-Scraper
    ```

3.  **Build and run the project:**
    ```sh
    cargo run --release
    ```

The scraper will start, and you will see progress printed to the console:

```
- Processing Pages 50000 to 1 (Total Pages: 50000)

[50000/50000] - Page: 50000; Response: 200 OK
[49999/50000] - Page: 49999; Response: 200 OK
...
```

The scraped data will be saved in the PostgreSQL `question_data` table, and the last processed page number will be stored in `output/LastPage.txt`. Run the command again to continue scraping if interrupted.

## Output Files

-   **PostgreSQL Database**: `question_data` table in the `stackoverflow_data` database containing all scraped questions.
-   **output/LastPage.txt**: Contains the last successfully processed page number (used to resume scraping).
-   **output/LostPage.txt**: Contains page numbers that failed to fetch (one per line) for manual review or retry.

## Performance Optimization

The release build is optimized for performance with the following settings in `Cargo.toml`:

-   Optimization level 3
-   Thin LTO (Link Time Optimization)
-   Single codegen unit
-   Panic abort strategy
-   Symbol stripping

## License

This project is provided as-is for educational purposes. Please respect Stack Overflow's Terms of Service and robots.txt when using this scraper.
