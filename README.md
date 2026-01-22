# Stack Overflow Scraper

A simple and efficient asynchronous web scraper built in Rust to fetch all question titles and their corresponding IDs from Stack Overflow. The scraped data is stored in a SQLite database.

## Features

-   **Asynchronous Fetching**: Uses `tokio` and `reqwest` for non-blocking HTTP requests, allowing for efficient scraping.
-   **Modular Architecture**: Separates concerns into dedicated modules (`database.rs`, `fileops.rs`, `questionsvalue.rs`) for better code organization and maintainability.
-   **Polite Scraping**: Implements random delays (0.1-1.9 seconds) between requests to avoid overwhelming the server and respect rate limits.
-   **Resilient**: Handles network errors and non-successful HTTP responses gracefully, logging failed pages for manual review.
-   **Data Persistence**: Stores scraped data in a SQLite database (`output/stackoverflow.db`) with a unique constraint on question IDs.
-   **Duplicate Prevention**: Checks for existing question IDs in the database before insertion to avoid duplicate entries.
-   **Incremental Scraping**:
    -   Processes a configurable number of pages per run (default: 500 pages via `PAGES_PER_RUN` constant).
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
│       ├── database.rs          # Database-related functions
│       ├── fileops.rs           # File I/O operations
│       └── questionsvalue.rs    # HTML parsing for question data
├── output/                      # Created automatically
│   ├── stackoverflow.db         # SQLite database with scraped questions
│   ├── LastPage.txt             # Tracks the last processed page number
│   └── LostPage.txt             # Logs failed page fetches
├── Cargo.toml                  # Project dependencies and configuration
└── README.md                   # This file
```

## How It Works

1.  **Initialization**: The scraper creates an `output` directory, initializes a SQLite database, and creates the `stackoverflow_questions` table if it doesn't exist.

2.  **Metadata Fetch**: It makes an initial request to Stack Overflow to determine the total number of questions and calculates the total number of pages to scrape (50 questions per page).

3.  **Resume from Last Session**: It reads `output/LastPage.txt` to find the last processed page and resumes scraping from there. If the file doesn't exist or is empty, it starts from the last page (newest questions).

4.  **Incremental Scraping Loop**: It processes up to `PAGES_PER_RUN` pages (default: 500) in reverse chronological order:
    -   For each page, it sends an HTTP GET request with a `User-Agent` header.
    -   Adds a random delay (0.1-1.9 seconds) between requests to be a polite scraper.
    -   Uses the `questionsvalue` module to parse the HTML response and extract question titles and IDs.
    -   Checks for duplicate question IDs in the database before insertion using functions from the `database` module.

5.  **Data Storage**: For each question, it checks if the ID already exists in the database. If not, it inserts the new question data (ID and title). After successfully processing a page, it updates `output/LastPage.txt` using functions from the `fileops` module.

6.  **Error Handling**: Failed page fetches are logged to `output/LostPage.txt` with their page numbers for manual investigation or retry.

7.  **Completion**: The process completes after processing the configured number of pages. Run the scraper again to continue from the next batch.

## Dependencies

The project relies on the following Rust crates:

-   `tokio` (v1.49.0): Asynchronous runtime with full features enabled.
-   `reqwest` (v0.13.1): HTTP client for making web requests.
-   `scraper` (v0.25.0): HTML parsing and CSS selector matching.
-   `rusqlite` (v0.38.0): SQLite database interaction with bundled SQLite.
-   `rand` (v0.9.2): Random number generation for delays.

## Database Schema

```sql
CREATE TABLE stackoverflow_questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    q_id INTEGER NOT NULL UNIQUE,      -- Stack Overflow question ID
    question TEXT NOT NULL              -- Question title
);
```

## Configuration

You can modify the following constants in `src/main.rs`:

-   `PAGES_PER_RUN` (default: 500) - Number of pages to process per execution.
-   `USER_AGENT` - Browser user agent string sent with requests.
-   `DB_PATH` - Path to the SQLite database file.

## How to Run

1.  **Clone the repository:**
    ```sh
    git clone <repository-url>
    cd Stackoverflow-Scraper
    ```

2.  **Build and run the project:**
    ```sh
    cargo run --release
    ```

The scraper will start, and you will see progress printed to the console:

```
- Processing Pages 50000 to 49501

[001/500] - Page: 50000; Response: 200 OK
[002/500] - Page: 49999; Response: 200 OK
...
```

The scraped data will be saved in `output/stackoverflow.db`, and the last processed page number will be stored in `output/LastPage.txt`. Run the command again to continue scraping the next batch of pages.

## Output Files

-   **output/stackoverflow.db**: SQLite database containing all scraped questions.
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
