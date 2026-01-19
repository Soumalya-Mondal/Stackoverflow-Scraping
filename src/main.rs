// -----------------------------
// External crate imports
// -----------------------------
// reqwest::Client is used to perform HTTP requests (GET pages from Stack Overflow).
use reqwest::Client;

// rand::Rng provides random number generation utilities.
// Here it's used to randomize delay between requests (polite scraping).
use rand::Rng;

// tokio::time is used because this is an async program.
// sleep + Duration help implement async delays between requests.
use tokio::time::{sleep, Duration};

// scraper provides HTML parsing utilities similar to BeautifulSoup (Rust ecosystem).
// Html is the parsed document; Selector is a CSS selector parser.
use scraper::{Html, Selector};

// std::fs is used for filesystem operations (create directories, create files, append).
use std::fs;

// csv::Writer is used to write output in CSV format.
use csv::Writer;

// std::io::Write provides low-level write APIs (used for writing BOM and LostPage entries).
use std::io::Write;

// std::io::Read for reading LastPage.txt
use std::io::Read;

// -----------------------------
// Constants (configuration)
// -----------------------------
// Base URL for StackOverflow questions listing.
const BASE_URL: &str = "https://stackoverflow.com/questions";

// Selector that points to the main block that contains the list of questions.
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";

// Selector that contains metadata about total question count.
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";

// User-Agent header to mimic a real browser (helps avoid simplistic blocks).
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

// Each question on list page is represented by this container.
const QUESTION_BLOCK_SELECTOR: &str = "div.s-post-summary.js-post-summary";

// Selector for question title text node.
const TITLE_SELECTOR: &str = "h3.s-post-summary--content-title a span[itemprop='name']";

// Selector for the question hyperlink (contains href).
const LINK_SELECTOR: &str = "h3.s-post-summary--content-title a.s-link";

// Maximum pages to process per run
const PAGES_PER_RUN: u64 = 10;

// -----------------------------
// Data model for extracted rows
// -----------------------------
// Represents one CSV row: question title, extracted question ID, and page number.
#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    id: u64,
}

// -----------------------------
// LastPage tracking helpers
// -----------------------------
// Reads the last processed page number from LastPage.txt
// Returns 0 if file doesn't exist or can't be read (start from beginning)
fn read_last_page() -> u64 {
    match fs::File::open("output/LastPage.txt") {
        Ok(mut file) => {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                content
                    .trim()
                    .parse()
                    .unwrap_or(0)
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

// Writes the current page number to LastPage.txt
fn write_last_page(page: u64) {
    if let Ok(mut file) = fs::File::create("output/LastPage.txt") {
        let _ = writeln!(file, "{}", page);
    }
}

// -----------------------------
// HTML parsing helper
// -----------------------------
// Parses a page HTML and extracts question title + question ID + page number.
// Returns Vec<QuestionRow> for downstream CSV writing.
fn parse_questions_views_and_links(page_html: &str) -> Vec<QuestionRow> {
    // Parse raw HTML string into a structured DOM document.
    let doc = Html::parse_document(page_html);

    // Compile CSS selectors once per call for efficiency and readability.
    let required_block_sel = Selector::parse(REQUIRED_BLOCK_SELECTOR).unwrap();
    let question_block_sel = Selector::parse(QUESTION_BLOCK_SELECTOR).unwrap();
    let title_sel = Selector::parse(TITLE_SELECTOR).unwrap();
    let link_sel = Selector::parse(LINK_SELECTOR).unwrap();

    // Collected question results for this page.
    let mut results = Vec::new();

    // Ensure the required page container exists; otherwise return empty results.
    // This prevents panics when HTML layout changes or request returns unexpected content.
    let required_block = match doc.select(&required_block_sel).next() {
        Some(b) => b,
        None => return results,
    };

    // Loop through each question summary block in the required container.
    for q in required_block.select(&question_block_sel) {
        // ---- Title ----
        // Extract title text, trim whitespace, default to empty string if missing.
        let title: String = q
            .select(&title_sel)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Skip invalid entries (defensive coding when structure changes).
        if title.is_empty() {
            continue;
        }

        // ---- Link (href) ----
        // Usually relative: "/questions/...."
        // Extract href attribute from the anchor tag.
        let href: String = q
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("")
            .to_string();

        // Extract ID from href (e.g., "/questions/79870378/..." -> 79870378)
        // Splits by '/' and reads the third token which is the numeric question id.
        let id: u64 = href
            .split('/')
            .nth(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Push a strongly typed row into results.
        results.push(QuestionRow { title, id });
    }

    // Return all extracted question rows from this page.
    results
}

// -----------------------------
// Program entry point (async)
// -----------------------------
// Uses tokio runtime to allow async HTTP calls and async delays.
#[tokio::main]
async fn main() {
    // Create a reusable HTTP client (connection pooling, better performance).
    let web_client = Client::new();

    // Create output directory if it doesn't exist
    // Ensures CSV and lost-page logs can be written safely.
    fs::create_dir_all("output")
        .unwrap();

    // Create the questions subdirectory if it doesn't exist
    fs::create_dir_all("output/questions")
        .unwrap();

    // Delete existing LostPage.txt file if present
    if fs::metadata("output/LostPage.txt").is_ok() {
        fs::remove_file("output/LostPage.txt")
            .unwrap();
    }

    // Read the last processed page (resume capability)
    let last_processed_page = read_last_page();
    println!("Last processed page: {}", last_processed_page);

    // Fetch first page to compute total pages
    // We need total question count to determine how many pages to crawl.
    let page_response = web_client
        .get(BASE_URL)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .unwrap();

    // Convert HTTP response body into text (HTML).
    let whole_page_content: String = page_response.text().await.unwrap();

    // Parse first page HTML to extract total number of items.
    let whole_html_parse_document = Html::parse_document(&whole_page_content);

    // Select the required questions block (main container).
    let required_block_selector = Selector::parse(REQUIRED_BLOCK_SELECTOR).unwrap();
    let required_block_element = whole_html_parse_document
        .select(&required_block_selector)
        .next()
        .unwrap();

    // Select the meta node containing total numberOfItems.
    let total_question_selector = Selector::parse(TOTAL_QUESTION_SELECTOR).unwrap();
    let total_question_element = required_block_element
        .select(&total_question_selector)
        .next()
        .unwrap();

    // Extract "content" attribute and parse to usize.
    let total_question_count: usize = total_question_element
        .value()
        .attr("content")
        .unwrap()
        .parse()
        .unwrap();

    // StackOverflow uses pagesize=50; compute how many pages exist.
    // div_ceil ensures partial pages count as one full page.
    let total_pages_count: u64 = total_question_count.div_ceil(50) as u64;

    // Calculate starting page (last processed + 1, but in reverse order)
    // Since we iterate in reverse, we need to calculate the correct starting point
    let starting_page = if last_processed_page == 0 {
        total_pages_count // Start from the last page if never run
    } else if last_processed_page == 1 {
        println!("All pages have been processed!");
        return;
    } else {
        last_processed_page - 1 // Continue from where we left off
    };

    // Calculate ending page (process up to 200 pages)
    let ending_page = if starting_page > PAGES_PER_RUN {
        starting_page - PAGES_PER_RUN + 1
    } else {
        1 // Stop at page 1
    };

    println!("Processing pages {} to {} (total {} pages)", starting_page, ending_page, starting_page - ending_page + 1);

    // Iterate pages in reverse order (last page to first page).
    let mut page_count: u64 = 1;
    for page in (ending_page..=starting_page).rev() {
        // Build URL for each page with fixed pagesize=50.
        let url: String = format!("{}?page={}&pagesize=50", BASE_URL, page);

        // Random polite delay to reduce chances of throttling / blocking.
        // The range (0.1..=1.9) seconds mimics human browsing.
        sleep(Duration::from_secs_f64(rand::rng().random_range(0.1..=1.9)))
            .await;

        // Perform the HTTP GET request and handle transport failures gracefully.
        let resp = match web_client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{} - Failed to fetch page {}: {}", page_count, page, e);
                continue;
            }
        };

        // Add terminal output for successful fetches
        println!("{:06} - PAGE: {}; RESPONSE: {}", page_count, page, resp.status());

        // If server returns non-2xx response, log it and save the page number.
        if !resp.status().is_success() {
            eprintln!("{} - Failed page {}: status {}", page_count, page, resp.status());

            // Append failed page info to output/LostPage.txt for later retry/debugging.
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("output/LostPage.txt")
                .unwrap();
            writeln!(file, "{}", page).unwrap();
            continue;
        }

        // Read response body to HTML string and handle body read errors.
        let html: String = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{} - Failed to read HTML page {}: {}", page_count, page, e);
                continue;
            }
        };

        // Extract question rows from this page using the parser helper.
        let items = parse_questions_views_and_links(&html);
        if items.is_empty() {
            // If parsing finds nothing, skip writing (page layout change or empty page).
            continue;
        }

        // Create a new CSV file for this page
        let csv_path: String = format!("output/questions/{}.csv", page);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&csv_path)
            .unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap(); // UTF-8 BOM
        let mut writer = Writer::from_writer(file);

        // Write CSV header row.
        writer.write_record(["Question", "ID"]).unwrap();

        // Write each extracted row to this page's CSV.
        // Convert numeric values to string because CSV writer expects string slices.
        for item in items {
            writer.write_record(&[
                item.title,
                item.id.to_string(),
            ]).unwrap();
        }

        // Flush buffered output to disk to ensure CSV is complete.
        writer.flush().unwrap();

        // Save the current page as the last successfully processed page
        write_last_page(page);

        // Increment page counter for logging.
        page_count += 1;
    }

    println!("\nCompleted processing {} pages. Last page processed: {}", page_count - 1, ending_page);
}
