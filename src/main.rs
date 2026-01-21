// ============================================================================
// Importing External Crates
// ============================================================================
use reqwest::Client;
use rand::Rng;
use tokio::time::{sleep, Duration};
use scraper::{Html, Selector};
use std::fs;
use rusqlite::{Connection, params};
use std::io::Write;

// ============================================================================
// Constants
// ============================================================================
const BASE_URL: &str = "https://stackoverflow.com/questions";
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
const QUESTION_BLOCK_SELECTOR: &str = "div.s-post-summary.js-post-summary";
const TITLE_SELECTOR: &str = "h3.s-post-summary--content-title a span[itemprop='name']";
const LINK_SELECTOR: &str = "h3.s-post-summary--content-title a.s-link";
const DB_PATH: &str = "output/stackoverflow.db";
const PAGES_PER_RUN: u64 = 10;

// ============================================================================
// Data Structures
// ============================================================================
#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    id: i64,
}

// ============================================================================
// Initialize Database Function
// ============================================================================
fn table_exists(conn: &Connection, table_name: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?1"
    )?;
    let exists = stmt.exists([table_name])?;
    Ok(exists)
}

fn init_database(conn: &Connection) -> rusqlite::Result<()> {
    if !table_exists(conn, "stackoverflow_questions")? {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stackoverflow_questions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                page_no INTEGER NOT NULL,
                q_id INTEGER NOT NULL UNIQUE,
                question TEXT NOT NULL
            )",
            [],
        )?;
    }
    Ok(())
}

// ============================================================================
// Parse Questions, Views And Links From HTML Function
// ============================================================================
fn parse_questions_views_and_links(page_html: &str) -> Vec<QuestionRow> {
    let doc = Html::parse_document(page_html);

    let required_block_sel = Selector::parse(REQUIRED_BLOCK_SELECTOR).unwrap();
    let question_block_sel = Selector::parse(QUESTION_BLOCK_SELECTOR).unwrap();
    let title_sel = Selector::parse(TITLE_SELECTOR).unwrap();
    let link_sel = Selector::parse(LINK_SELECTOR).unwrap();

    let mut results = Vec::new();

    let required_block = match doc.select(&required_block_sel).next() {
        Some(b) => b,
        None => return results,
    };

    for q in required_block.select(&question_block_sel) {
        let title: String = q
            .select(&title_sel)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.is_empty() {
            continue;
        }

        let href: String = q
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("")
            .to_string();

        let id: i64 = href
            .split('/')
            .nth(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        results.push(QuestionRow { title, id });
    }

    results
}

// ============================================================================
// Log Message To File Function
// ============================================================================
fn log_to_file(message: &str) {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("output/ScrapingLog.txt")
        .unwrap();
    writeln!(file, "{}", message).unwrap();
}

// ============================================================================
// Get Last Processed Page From Database Function
// ============================================================================
fn get_last_processed_page(conn: &Connection) -> rusqlite::Result<i64> {
    let mut stmt = conn.prepare("SELECT MIN(page_no) FROM stackoverflow_questions")?;
    let last_page: Option<i64> = stmt.query_row([], |row| row.get(0))?;
    Ok(last_page.unwrap_or(0))
}

// ============================================================================
// Main Async Function
// ============================================================================
#[tokio::main]
async fn main() {
    let web_client = Client::new();

    fs::create_dir_all("output")
        .unwrap();

    let conn = Connection::open(DB_PATH)
        .expect("Failed to open database");

    init_database(&conn)
        .expect("Failed to initialize database");

    let page_response = web_client
        .get(BASE_URL)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .unwrap();

    let whole_page_content: String = page_response.text().await.unwrap();

    let whole_html_parse_document = Html::parse_document(&whole_page_content);

    let required_block_selector = Selector::parse(REQUIRED_BLOCK_SELECTOR).unwrap();
    let required_block_element = whole_html_parse_document
        .select(&required_block_selector)
        .next()
        .unwrap();

    let total_question_selector = Selector::parse(TOTAL_QUESTION_SELECTOR).unwrap();
    let total_question_element = required_block_element
        .select(&total_question_selector)
        .next()
        .unwrap();

    let total_question_count: usize = total_question_element
        .value()
        .attr("content")
        .unwrap()
        .parse()
        .unwrap();

    let total_pages_count: u64 = total_question_count.div_ceil(50) as u64;

    let last_processed_page = get_last_processed_page(&conn)
        .expect("Failed to get last processed page") as u64;

    let start_page = if last_processed_page == 0 {
        total_pages_count
    } else {
        last_processed_page + 1
    };

    let end_page = if start_page > PAGES_PER_RUN {
        start_page - PAGES_PER_RUN + 1
    } else {
        1
    };

    println!("- Processing Pages {} to {}\n", end_page, start_page);

    let mut page_count: u8 = 1;
    for page in (end_page..=start_page).rev() {
        let url: String = format!("{}?page={}&pagesize=50", BASE_URL, page);

        sleep(Duration::from_secs_f64(rand::rng().random_range(0.1..=1.9)))
            .await;

        let resp = match web_client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed To Fetch Page {}: {}", page, e);
                continue;
            }
        };

        println!("[{:03}/{:03}] - Page: {}; Response: {}", page_count, PAGES_PER_RUN, page, resp.status());

        if !resp.status().is_success() {
            eprintln!("Failed Page {}: Status {}", page, resp.status());

            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("output/LostPage.txt")
                .unwrap();
            writeln!(file, "{}", page).unwrap();
            continue;
        }

        let html: String = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed To Read HTML Page {}: {}", page, e);
                continue;
            }
        };

        let items = parse_questions_views_and_links(&html);
        if items.is_empty() {
            continue;
        }

        for item in items {
            // Check if "q_id" already exists
            let mut stmt = conn.prepare("SELECT id FROM stackoverflow_questions WHERE q_id = ?1")
                .expect("Failed To Prepare Statement");
            let existing_id: Result<i64, _> = stmt.query_row([item.id], |row| row.get(0));

            if let Ok(prev_id) = existing_id {
                log_to_file(&format!("Question-ID: {:06} Already Exists With Old-ID: {:06}", item.id, prev_id));
                continue;
            }

            let title_utf8 = String::from_utf8_lossy(item.title.as_bytes()).to_string();

            match conn.execute(
                "INSERT INTO stackoverflow_questions (page_no, q_id, question) VALUES (?1, ?2, ?3)",
                params![page as i64, item.id, &title_utf8],
            ) {
                Ok(_) => {},
                Err(e) => eprintln!("Failed To Insert Question {}: {}", item.id, e),
            }
        }
        page_count += 1;
    }

    println!("\n- Completed Processing Pages {} to {}", end_page, start_page);
}