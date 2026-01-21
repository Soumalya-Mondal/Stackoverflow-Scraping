# Stackoverflow Scraper - Line by Line Explanation

## Imports (Lines 1-9)
- `use reqwest::Client;` - HTTP client for making web requests.
- `use rand::Rng;` - Random number generation for delays.
- `use tokio::time::{sleep, Duration};` - Async sleep functionality.
- `use scraper::{Html, Selector};` - HTML parsing and CSS selector matching.
- `use std::fs;` - File system operations (for creating output directory and log file).
- `use rusqlite::{Connection, params};` - SQLite database connection and operations.
- `use std::io::Write;` - Basic I/O operations (for logging).

## Constants (Lines 11-19)
- `BASE_URL` - Stack Overflow questions page URL.
- `REQUIRED_BLOCK_SELECTOR` - CSS selector for the main questions container.
- `TOTAL_QUESTION_SELECTOR` - CSS selector for total question count metadata.
- `USER_AGENT` - Browser user agent string to avoid blocking.
- `QUESTION_BLOCK_SELECTOR` - CSS selector for individual question blocks.
- `TITLE_SELECTOR` - CSS selector for question titles.
- `LINK_SELECTOR` - CSS selector for question links/hrefs.
- `DB_PATH` - Path to the SQLite database file.
- `PAGES_PER_RUN` - Maximum number of pages to scrape per execution (default: 10).

## Data Structure (Lines 21-25)
```rust
#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    id: i64,
}
```
Represents a single question with its title and Stack Overflow ID.

## Database Functions (Lines 27-57)

### Function: `table_exists()`
- Checks if a table with a given name exists in the SQLite database.

### Function: `init_database()`
- Initializes the database.
- If the `stackoverflow_questions` table doesn't exist, it creates it with columns: `id` (primary key), `page_no`, `q_id` (unique question ID), and `question` (title).

### Function: `get_last_processed_page()`
- Queries the database to retrieve the minimum (lowest) page number that has been processed.
- Returns 0 if no pages have been processed yet (indicating this is the first run).
- Used to determine where to resume scraping on subsequent runs.

## Function: `parse_questions_views_and_links()` (Lines 59-101)
Extracts questions from an HTML page string:
1. Parses the HTML into a DOM document.
2. Compiles CSS selectors for efficiency.
3. Finds the main questions container.
4. Iterates through each question block:
   - Extracts the title text.
   - Extracts the href attribute from the link.
   - Parses the question ID from the href (e.g., `/questions/12345/...` → `12345`).
   - Creates a `QuestionRow` and adds it to a results vector.
5. Returns a `Vec<QuestionRow>` of all extracted questions.

## Function: `log_to_file()` (Lines 103-111)
- Appends a given message to `output/ScrapingLog.txt`.
- Used for logging when a duplicate question ID is found.

## Main Function (Lines 113-227)

### Setup (Lines 116-125)
- Creates an HTTP client.
- Creates the `output/` directory.
- Opens a connection to the SQLite database at `DB_PATH`.
- Calls `init_database()` to ensure the table exists.

### Fetch Metadata (Lines 127-153)
- Makes a GET request to the Stack Overflow base URL to get the first page.
- Parses the HTML to find the total number of questions from a metadata tag.
- Calculates the total number of pages by dividing the total questions by 50 (questions per page).

### Calculate Page Range (Lines 155-169)
- Calls `get_last_processed_page()` to retrieve the minimum page number already in the database.
- If no data exists (returns 0), `start_page` is set to `total_pages_count` (the last page).
- Otherwise, `start_page` is set to `last_processed_page + 1` (resuming from the next unprocessed page).
- `end_page` is calculated as `start_page - PAGES_PER_RUN + 1`, but never goes below 1.
- This ensures exactly `PAGES_PER_RUN` pages are processed per run (or fewer if reaching page 1).

### Process Pages in Reverse (Lines 171-227)
Iterates through pages from `start_page` down to `end_page` in reverse order:

1.  **Build URL** - Formats the URL for the current page number with `pagesize=50`.

2.  **Add Random Delay** - Sleeps for a random duration between 0.1 and 1.9 seconds to be a polite scraper.

3.  **Fetch Page** - Makes an HTTP GET request with the `User-Agent` header.
    - If there's a network error, it prints the error and continues to the next page.

4.  **Log Response Status** - Prints the current page number, total pages processed in this run, and the HTTP status of the response.

5.  **Handle Failed Requests** - If the HTTP status is not successful:
    - Prints an error.
    - Appends the failed page number to `output/LostPage.txt` for manual review.
    - Continues to the next page.

6.  **Parse Questions** - Reads the response body as text and calls `parse_questions_views_and_links()` to extract question data.

7.  **Insert into Database** - For each extracted question:
    - It first checks if the question ID (`q_id`) already exists in the database to prevent duplicates.
    - If it exists, it logs the duplicate detection to `ScrapingLog.txt` and skips it.
    - If it's a new question, it inserts the page number, question ID, and title into the `stackoverflow_questions` table.

### Final Summary (Line 229)
Prints a completion message indicating which page range was processed.

## Execution Flow Summary
1.  Initialize database connection and create table if needed.
2.  Fetch the first page of Stack Overflow to determine the total number of pages.
3.  Query the database to find the last processed page and calculate the range to process.
4.  Iterate through the calculated page range in reverse order (up to `PAGES_PER_RUN` pages).
5.  For each page:
    - Fetch the HTML with a polite delay.
    - Parse questions from the HTML.
    - For each question, check for existence in the database.
    - Insert new questions into the SQLite database.
6.  Log errors and duplicate entries.
7.  Report completion with the processed page range.
8.  On the next execution, resume from where the previous run left off.