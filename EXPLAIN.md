# Stackoverflow Scraper - Line by Line Explanation

## Imports (Lines 1-7)
- `use reqwest::Client;` - HTTP client for making web requests
- `use rand::Rng;` - Random number generation for delays
- `use tokio::time::{sleep, Duration};` - Async sleep functionality
- `use scraper::{Html, Selector};` - HTML parsing and CSS selector matching
- `use std::fs;` - File system operations
- `use csv::Writer;` - CSV file writing
- `use std::io::{Write, Read};` - Basic I/O operations

## Constants (Lines 9-17)
- `BASE_URL` - Stack Overflow questions page URL
- `REQUIRED_BLOCK_SELECTOR` - CSS selector for the questions container
- `TOTAL_QUESTION_SELECTOR` - CSS selector for total question count metadata
- `USER_AGENT` - Browser user agent string to avoid blocking
- `QUESTION_BLOCK_SELECTOR` - CSS selector for individual question blocks
- `TITLE_SELECTOR` - CSS selector for question titles
- `LINK_SELECTOR` - CSS selector for question links/hrefs
- `PAGES_PER_RUN` - Maximum pages to process per execution (100)

## Data Structure (Lines 19-22)
```rust
#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    id: u64,
}
```
Represents a single question with title and ID for CSV output.

## Function: read_last_page() (Lines 24-37)
Reads the last processed page number from `output/LastPage.txt`:
- Opens the file if it exists
- Parses the content as a u64 number
- Returns 0 if file doesn't exist (starts fresh)
- Returns 0 if parsing fails

## Function: write_last_page(page: u64) (Lines 39-42)
Writes the current page number to `output/LastPage.txt`:
- Creates or overwrites the file
- Writes the page number for resuming later

## Function: parse_questions_views_and_links() (Lines 44-83)
Extracts questions from HTML page:
1. Parses HTML string into a DOM document
2. Compiles CSS selectors for efficiency
3. Finds the main questions container
4. Iterates through each question block:
   - Extracts title text
   - Skips empty titles
   - Extracts href attribute from link
   - Parses question ID from href (e.g., `/questions/12345/...` → `12345`)
   - Creates QuestionRow and adds to results
5. Returns Vec of all extracted questions

## Main Function (Lines 85-200)

### Setup (Lines 88-94)
- Creates HTTP client for connection pooling
- Creates `output/` directory
- Creates `output/questions-id/` directory for CSV files
- Reads last processed page from file

### Fetch First Page Metadata (Lines 96-118)
- Makes GET request to Stack Overflow base URL
- Parses HTML response
- Finds total question count from metadata
- Calculates total pages (dividing by 50, the page size)

### Calculate Page Range (Lines 120-139)
- If never run before: start from last page
- If already processed all pages: exit
- Otherwise: start from previous page - 1
- Calculate ending page based on `PAGES_PER_RUN` limit
- Prints processing range to console

### Process Pages in Reverse (Lines 141-200)
For each page from `ending_page` down to `starting_page`:

1. **Build URL** - Format with page number and pagesize=50

2. **Add Random Delay** - Sleep 0.1 to 1.9 seconds (polite scraping)

3. **Fetch Page** - Make HTTP GET request with User-Agent header
   - If network error: print error and continue to next page

4. **Log Response Status** - Print page number and HTTP status code

5. **Handle Failed Requests** - If status code is not successful:
   - Print error message
   - Append page number to `output/LostPage.txt` for retry
   - Continue to next page

6. **Read Response Body** - Convert response to HTML string
   - If read error: print error and continue

7. **Parse Questions** - Call parser function to extract questions
   - If no questions found: skip to next page

8. **Write CSV File** - Create `output/questions-id/{page}.csv`:
   - Write UTF-8 BOM (3 bytes: 0xEF, 0xBB, 0xBF)
   - Create CSV writer
   - Write header row: "Question", "ID"
   - Write each question with title and ID
   - Flush to disk

9. **Update Last Page** - Save current page number for resume

10. **Increment Counter** - Track processed pages

### Final Summary (Line 202)
Prints total pages processed and last page number processed.

## Execution Flow Summary
1. Resume from last known page or start from the end
2. Fetch Stack Overflow to get total question count
3. Calculate page range to process (max 100 pages per run)
4. For each page (in reverse order):
   - Fetch HTML with polite delay
   - Parse questions from HTML
   - Save to individual CSV file
   - Track last page in case of interruption
5. Report completion statistics