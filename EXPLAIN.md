# Stackoverflow Scraper - Line by Line Explanation

## Project Structure
The project is organized into the following modules:
- `main.rs` - The entry point of the application, responsible for orchestrating the scraping process.
- `supportscript/` - A directory containing helper modules.
  - `mod.rs` - Declares the modules within `supportscript`.
  - `database.rs` - Handles all database interactions, including initialization and data insertion.
  - `fileops.rs` - Manages file operations, such as reading and writing the last processed page number.
  - `questionsvalue.rs` - Contains the logic for parsing HTML and extracting question data.

## main.rs

### Imports (Lines 1-9)
- `use reqwest::Client;` - HTTP client for making web requests.
- `use rand::Rng;` - Random number generation for delays.
- `use tokio::time::{sleep, Duration};` - Async sleep functionality.
- `use scraper::{Html, Selector};` - HTML parsing and CSS selector matching.
- `use std::fs;` - File system operations (for creating output directory and log file).
- `use std::io::Write;` - Basic I/O operations (for file writing).

### Module Declarations (Lines 11-14)
- `mod supportscript;` - Declares the `supportscript` module.
- `use supportscript::{parse_questions, connect_database, init_database, get_last_processed_page_from_file, save_last_page_to_file};` - Imports functions from the support script modules.

### Constants (Lines 16-23)
- `BASE_URL` - Stack Overflow questions page URL.
- `REQUIRED_BLOCK_SELECTOR` - CSS selector for the main questions container.
- `TOTAL_QUESTION_SELECTOR` - CSS selector for total question count metadata.
- `USER_AGENT` - Browser user agent string to avoid blocking.

### Main Async Function (Lines 25-220)

#### Setup (Lines 28-39)
- Hardcoded database connection parameters:
  - `host` = "localhost"
  - `port` = "5432"
  - `database_name` = "stackoverflow_data"
  - `database_user` = "soumalya"
  - `password` = "Soumalya@1996"
- Creates an HTTP client using `reqwest::Client::new()`.
- Creates the `output/` directory if it doesn't exist.
- Calls `connect_database()` to establish a PostgreSQL connection.
- Calls `init_database()` to ensure the `question_data` table exists with the correct schema.

#### Fetch Metadata (Lines 41-69)
- Makes a GET request to the Stack Overflow base URL to get the first page.
- Parses the HTML to find the total number of questions from a metadata tag.
- Calculates the total number of pages by dividing the total questions by 50 (questions per page) using `div_ceil()`.

#### Calculate Page Range (Lines 71-76)
- Calls `get_last_processed_page_from_file()` to retrieve the last page number processed.
- If no data exists (returns 0), `start_page` is set to `total_pages_count` (scraping from the newest questions).
- Otherwise, `start_page` is set to `last_processed_page - 1` (resuming from the previous unprocessed page).
- The loop processes all pages from `start_page` down to page 1.
- This processes all remaining pages in a single run.

#### Process Pages in Reverse (Lines 78-165)
Iterates through pages from 1 up to `start_page` in reverse order:

1.  **Build URL** - Formats the URL for the current page number with `pagesize=50`.

2.  **Add Random Delay** - Sleeps for a random duration between 0.1 and 1.9 seconds using `rand::rng().random_range()` to be a polite scraper and avoid rate limiting.

3.  **Fetch Page** - Makes an HTTP GET request with the `User-Agent` header.
    - If there's a network error, it prints the error to stderr and continues to the next page.

4.  **Log Response Status** - Prints the current page number and total pages count along with the HTTP status code in format `[current_page/total_pages]`.

5.  **Handle Failed Requests** - If the HTTP status is not successful:
    - Prints an error message to stderr.
    - Appends the failed page number to `output/LostPage.txt` for manual review later.
    - Continues to the next page.

6.  **Parse Questions** - Reads the response body as text and calls `parse_questions()` from the `questionsvalue` module to extract question data.
    - If no questions are found (empty vector), continues to the next page.

7.  **Insert into Database** - For each extracted question:
    - Executes a SELECT query to check if the question ID (`q_id`) already exists in the `question_data` table.
    - If it exists, silently skips the question to avoid duplicates.
    - If it's a new question:
      - Converts the title to valid UTF-8 using `String::from_utf8_lossy()`.
      - Converts timestamp components to i32 for PostgreSQL compatibility.
      - Inserts the question ID, title (as `titel`), and timestamp components (`q_year`, `q_month`, `q_day`, `q_hours`, `q_min`, `q_sec`) into the `question_data` table using PostgreSQL parameterized queries ($1, $2, etc.).
      - The `row_inserted_at` column is automatically populated with the current timestamp.
      - Prints an error to stderr if the insertion fails.

8.  **Save Progress** - After processing all questions on a page, calls `save_last_page_to_file()` to update the last processed page number in `output/LastPage.txt`.

#### Final Summary (Line 167)
Prints a completion message indicating which page range was processed.

## supportscript/database.rs

### Function: `connect_database()`
- Connects to a PostgreSQL database using the provided connection parameters.
- Parameters: `host`, `port`, `database_name`, `database_user`, `password`.
- Spawns the connection in a background task.
- Returns a `Client` for database operations.

### Function: `init_database()`
- Initializes the PostgreSQL database.
- Creates the `question_data` table if it doesn't exist with columns: 
  - `id` (SERIAL PRIMARY KEY - auto-incrementing)
  - `q_id` (BIGINT NOT NULL UNIQUE - Stack Overflow question ID)
  - `titel` (TEXT - question title)
  - `q_year`, `q_month`, `q_day`, `q_hours`, `q_min`, `q_sec` (INTEGER NOT NULL)
  - `row_inserted_at` (TIMESTAMPTZ DEFAULT NOW() - automatic insertion timestamp)

## supportscript/fileops.rs

### Function: `get_last_processed_page_from_file()`
- Reads the last processed page number from `output/LastPage.txt`.
- Returns `0` if the file doesn't exist or cannot be parsed.

### Function: `save_last_page_to_file()`
- Saves the given page number to `output/LastPage.txt`, overwriting its content.
- Called after each page is successfully processed to track progress.

## supportscript/questionsvalue.rs

### Constants (Lines 6-10)
- `REQUIRED_BLOCK_SELECTOR` - CSS selector for the main questions container (`div#questions`).
- `QUESTION_BLOCK_SELECTOR` - CSS selector for individual question blocks.
- `TITLE_SELECTOR` - CSS selector for question titles.
- `LINK_SELECTOR` - CSS selector for question links/hrefs.
- `TIMESTAMP_SELECTOR` - CSS selector for the relative time element.

### Data Structure (Lines 12-23)
```rust
#[derive(Debug, Clone)]
pub struct QuestionRow {
    pub title: String,
    pub id: i64,
    pub q_year: u16,
    pub q_month: u8,
    pub q_day: u8,
    pub q_hour: u8,
    pub q_min: u8,
    pub q_sec: u8,
}
```
Represents a single question with its title, Stack Overflow ID, and timestamp components.

### Function: `parse_questions()` (Lines 25-81)
Extracts questions from an HTML page string:
1. Parses the HTML into a DOM document using `Html::parse_document()`.
2. Compiles CSS selectors for efficiency using `Selector::parse()`.
3. Finds the main questions container using the `REQUIRED_BLOCK_SELECTOR`.
   - Returns an empty vector if the container is not found.
4. Iterates through each question block within the container:
   - Extracts the title text, trims whitespace, and converts to String.
   - Skips questions with empty titles.
   - Extracts the href attribute from the question link.
   - Parses the question ID from the href (e.g., `/questions/12345/...` → `12345`).
   - Extracts the timestamp string from the `title` attribute of the `relativetime` span.
   - Calls `parse_timestamp()` to convert the timestamp string into its components.
   - Creates a `QuestionRow` and adds it to the results vector.
5. Returns a `Vec<QuestionRow>` of all extracted questions.

### Function: `parse_timestamp()` (Lines 83-111)
A helper function that parses a timestamp string (e.g., "2026-01-23 13:32:20") into a tuple of `(year, month, day, hour, min, sec)`. If parsing fails, it returns the current local time.

## Execution Flow Summary
1.  Connect to PostgreSQL database using hardcoded credentials and create/update the `question_data` table schema.
2.  Fetch the first page of Stack Overflow to determine the total number of pages.
3.  Read `output/LastPage.txt` to find the last processed page and calculate the starting page.
4.  Iterate through all pages from the starting page down to page 1 in reverse order.
5.  For each page:
    - Fetch the HTML with a polite random delay.
    - Parse questions and their timestamps from the HTML.
    - For each question, check for existence in the PostgreSQL database.
    - Insert new questions (including timestamp) into the `question_data` table.
    - The `row_inserted_at` column is automatically populated with the current timestamp.
    - Update `output/LastPage.txt` with the current page number.
6.  Log errors for failed page fetches to `output/LostPage.txt`.
7.  Report completion with the processed page range.
8.  On the next execution, resume from where the previous run left off.

## Key Design Decisions
- **PostgreSQL Database**: Uses PostgreSQL instead of SQLite for better scalability and production readiness.
- **Modular Architecture**: Separation of parsing logic into `questionsvalue.rs` allows for better code organization and testability.
- **Continuous Processing**: Processes all remaining pages from the last checkpoint down to page 1 in a single run.
- **Duplicate Prevention**: Database-level uniqueness constraint and query-based checking prevent duplicate entries.
- **Error Resilience**: Individual page failures don't stop the entire process; failed pages are logged for retry.
- **Polite Scraping**: Random delays between requests respect the target server's resources.
- **Automatic Timestamps**: The `row_inserted_at` column automatically tracks when each record was inserted.