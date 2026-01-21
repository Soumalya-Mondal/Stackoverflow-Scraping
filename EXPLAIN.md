# Stackoverflow Scraper - Line by Line Explanation

## Imports (Lines 1-9)
- `use reqwest::Client;` - HTTP client for making web requests.
- `use rand::Rng;` - Random number generation for delays.
- `use tokio::time::{sleep, Duration};` - Async sleep functionality.
- `use scraper::{Html, Selector};` - HTML parsing and CSS selector matching.
- `use std::fs;` - File system operations (for creating output directory and log file).
- `use rusqlite::{Connection, params};` - SQLite database connection and operations.
- `use std::io::Write;` - Basic I/O operations (for logging).

## Constants (Lines 11-18)
- `BASE_URL` - Stack Overflow questions page URL.
- `REQUIRED_BLOCK_SELECTOR` - CSS selector for the main questions container.
- `TOTAL_QUESTION_SELECTOR` - CSS selector for total question count metadata.
- `USER_AGENT` - Browser user agent string to avoid blocking.
- `QUESTION_BLOCK_SELECTOR` - CSS selector for individual question blocks.
- `TITLE_SELECTOR` - CSS selector for question titles.
- `LINK_SELECTOR` - CSS selector for question links/hrefs.
- `DB_PATH` - Path to the SQLite database file.

## Data Structure (Lines 20-24)
```rust
#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    id: i64,
}
```
Represents a single question with its title and Stack Overflow ID.

## Database Functions (Lines 26-47)

### Function: `table_exists()`
- Checks if a table with a given name exists in the SQLite database.

### Function: `init_database()`
- Initializes the database.
- If the `stackoverflow_questions` table doesn't exist, it creates it with columns: `id` (primary key), `page_no`, `q_id` (unique question ID), and `question` (title).

## Function: `parse_questions_views_and_links()` (Lines 49-91)
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

## Function: `log_to_file()` (Lines 93-101)
- Appends a given message to `output/ScrapingLog.txt`.
- Used for logging when a duplicate question ID is found.

## Main Function (Lines 103-201)

### Setup (Lines 106-115)
- Creates an HTTP client.
- Creates the `output/` directory.
- Opens a connection to the SQLite database at `DB_PATH`.
- Calls `init_database()` to ensure the table exists.

### Fetch Metadata (Lines 117-143)
- Makes a GET request to the Stack Overflow base URL to get the first page.
- Parses the HTML to find the total number of questions from a metadata tag.
- Calculates the total number of pages by dividing the total questions by 50 (questions per page).

### Process Pages in Reverse (Lines 145-201)
Iterates through every page from the last page down to the first:

1.  **Build URL** - Formats the URL for the current page number with `pagesize=50`.

2.  **Add Random Delay** - Sleeps for a random duration between 0.1 and 1.9 seconds to be a polite scraper.

3.  **Fetch Page** - Makes an HTTP GET request with the `User-Agent` header.
    - If there's a network error, it prints the error and continues to the next page.

4.  **Log Response Status** - Prints the current page number and the HTTP status of the response.

5.  **Handle Failed Requests** - If the HTTP status is not successful:
    - Prints an error.
    - Appends the failed page number to `output/LostPage.txt` for manual review.
    - Continues to the next page.

6.  **Parse Questions** - Reads the response body as text and calls `parse_questions_views_and_links()` to extract question data.

7.  **Insert into Database** - For each extracted question:
    - It first checks if the question ID (`q_id`) already exists in the database to prevent duplicates.
    - If it exists, it logs the duplicate detection to `ScrapingLog.txt` and skips it.
    - If it's a new question, it inserts the page number, question ID, and title into the `stackoverflow_questions` table.

### Final Summary (Line 203)
Prints a completion message after iterating through all pages.

## Execution Flow Summary
1.  Initialize database connection and create table if needed.
2.  Fetch the first page of Stack Overflow to determine the total number of pages.
3.  Iterate through all pages in reverse order (from last to first).
4.  For each page:
    - Fetch the HTML with a polite delay.
    - Parse questions from the HTML.
    - For each question, check for existence in the database.
    - Insert new questions into the SQLite database.
5.  Log errors and duplicate entries.
6.  Report completion.