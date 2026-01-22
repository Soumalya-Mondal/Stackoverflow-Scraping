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
// Module Declarations
// ============================================================================
mod supportscript;
use supportscript::{parse_questions, init_database, get_last_processed_page_from_file, save_last_page_to_file};

// ============================================================================
// Constants
// ============================================================================
const BASE_URL: &str = "https://stackoverflow.com/questions";
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
const DB_PATH: &str = "output/stackoverflow.db";
const PAGES_PER_RUN: u64 = 500;

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
    let last_processed_page = get_last_processed_page_from_file() as u64;

    let start_page = if last_processed_page == 0 { total_pages_count } else { last_processed_page + 1 };
    let end_page = std::cmp::max(1, start_page.saturating_sub(PAGES_PER_RUN - 1));

    println!("- Processing Pages {} to {}\n", start_page, end_page);

    let mut page_count: u16 = 1;
    let mut last_processed_page: u64;

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
                page_count += 1;
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
            page_count += 1;
            continue;
        }

        let html: String = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed To Read HTML Page {}: {}", page, e);
                page_count += 1;
                continue;
            }
        };

        let items = parse_questions(&html);
        if items.is_empty() {
            page_count += 1;
            continue;
        }

        for item in items {
            // Check if "q_id" already exists
            let mut stmt = conn.prepare("SELECT id FROM stackoverflow_questions WHERE q_id = ?1")
                .expect("Failed To Prepare Statement");
            let existing_id: Result<i64, _> = stmt.query_row([item.id], |row| row.get(0));

            if existing_id.is_ok() {
                continue;
            }

            let title_utf8 = String::from_utf8_lossy(item.title.as_bytes()).to_string();

            match conn.execute(
                "INSERT INTO stackoverflow_questions (q_id, question) VALUES (?1, ?2)",
                params![item.id, &title_utf8],
            ) {
                Ok(_) => {},
                Err(e) => eprintln!("Failed To Insert Question {}: {}", item.id, e),
            }
        }

        last_processed_page = page;
        save_last_page_to_file(last_processed_page);
        page_count += 1;
    }

    println!("\n- Completed Processing Pages {} to {}", start_page, end_page);
}