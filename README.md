# Stack Overflow Scraper

A simple and efficient asynchronous web scraper built in Rust to fetch all question titles and their corresponding IDs from Stack Overflow. The scraped data is stored in a SQLite database.

## Features

-   **Asynchronous Fetching**: Uses `tokio` and `reqwest` for non-blocking HTTP requests, allowing for efficient scraping.
-   **Polite Scraping**: Implements random delays between requests to avoid overwhelming the server.
-   **Resilient**: Handles network errors and non-successful HTTP responses gracefully.
-   **Data Persistence**: Stores scraped data in a SQLite database (`output/stackoverflow.db`).
-   **Duplicate Prevention**: Checks for existing question IDs in the database to avoid duplicate entries.
-   **Incremental Scraping**: Processes a configurable number of pages per run (default: 10 pages) and resumes from where it left off on subsequent runs.
-   **Logging**:
    -   Logs failed page fetches to `output/LostPage.txt`.
    -   Logs detected duplicate question IDs to `output/ScrapingLog.txt`.

## How It Works

1.  **Initialization**: The scraper creates an `output` directory, initializes a SQLite database, and creates the necessary table if it doesn't exist.
2.  **Metadata Fetch**: It makes an initial request to Stack Overflow to determine the total number of questions and calculates the total number of pages to scrape.
3.  **Resume from Last Session**: It queries the database to find the last processed page and resumes scraping from there (or starts from the last page if no prior data exists).
4.  **Incremental Scraping Loop**: It processes `PAGES_PER_RUN` pages (default: 10) in reverse order:
    -   For each page, it sends an HTTP GET request with a `User-Agent` header.
    -   It parses the HTML response to extract question titles and IDs.
5.  **Data Storage**: For each question, it checks if the ID already exists in the database. If not, it inserts the new question data.
6.  **Completion**: The process completes after processing the configured number of pages. Run the scraper again to continue from the next batch.

## Dependencies

The project relies on the following Rust crates:

-   `tokio`: For the asynchronous runtime.
-   `reqwest`: For making HTTP requests.
-   `scraper`: For parsing HTML and selecting elements with CSS selectors.
-   `rusqlite`: For SQLite database interaction.
-   `rand`: For generating random delays.

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

The scraper will start, and you will see progress printed to the console. The scraped data will be saved in `output/stackoverflow.db`. Run the command again to continue scraping the next batch of pages.
