// ============================================================================
// Importing External Crates
// ============================================================================
use reqwest::Client;
use rand::Rng;
use tokio::time::{sleep, Duration};
use scraper::{Html, Selector};
use std::fs;
use std::io::Write;
use dotenv::dotenv;
use std::env;

// ============================================================================
// Module Declarations
// ============================================================================
mod supportscript;
use supportscript::{parse_questions, connect_database, init_database, get_last_processed_page_from_file, save_last_page_to_file};

// ============================================================================
// Constants
// ============================================================================
const BASE_URL: &str = "https://stackoverflow.com/questions";
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

// ============================================================================
// Main Async Function
// ============================================================================
#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Get database connection parameters from environment variables
    let host = env::var("DATABASE_HOST").expect("DATABASE_HOST must be set");
    let port = env::var("DATABASE_PORT").expect("DATABASE_PORT must be set");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let database_user = env::var("DATABASE_USER").expect("DATABASE_USER must be set");
    let password = env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD must be set");

    let web_client = Client::new();

    fs::create_dir_all("output")
        .unwrap();

    let client = connect_database(&host, &port, &database_name, &database_user, &password)
        .await
        .expect("Failed to connect to database");

    init_database(&client)
        .await
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

    let start_page = if last_processed_page == 0 { total_pages_count } else { last_processed_page - 1 };

    println!("- Processing Pages {} to 1 (Total Pages: {})\n", start_page, total_pages_count);

    let mut last_processed_page: u64;
    let mut iteration: u64 = 1;

    for page in (1..=start_page).rev() {
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
                iteration += 1;
                continue;
            }
        };

        println!("[{}/{}] - Page: {}; Response: {}", iteration, total_pages_count, page, resp.status());

        if !resp.status().is_success() {
            eprintln!("Failed Page {}: Status {}", page, resp.status());

            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("output/LostPage.txt")
                .unwrap();
            writeln!(file, "{}", page).unwrap();
            iteration += 1;
            continue;
        }

        let html: String = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed To Read HTML Page {}: {}", page, e);
                continue;
            }
        };

        let items = parse_questions(&html);
        if items.is_empty() {
            continue;
        }

        for item in items {
            // Check if "q_id" already exists
            let existing = client.query(
                "SELECT id FROM question_data WHERE q_id = $1",
                &[&item.id],
            ).await;

            if let Ok(rows) = existing
                && !rows.is_empty() {
                    continue;
                }

            let title_utf8 = String::from_utf8_lossy(item.title.as_bytes()).to_string();
            
            let q_year = item.q_year as i32;
            let q_month = item.q_month as i32;
            let q_day = item.q_day as i32;
            let q_hours = item.q_hour as i32;
            let q_min = item.q_min as i32;
            let q_sec = item.q_sec as i32;

            match client.execute(
                "INSERT INTO question_data (q_id, q_title, q_year, q_month, q_day, q_hours, q_min, q_sec) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (q_id) DO NOTHING",
                &[&item.id, &title_utf8, &q_year, &q_month, &q_day, &q_hours, &q_min, &q_sec],
            ).await {
                Ok(_) => {},
                Err(e) => eprintln!("Failed To Insert Question {}: {}", item.id, e),
            }
        }

        last_processed_page = page;
        save_last_page_to_file(last_processed_page);
        iteration += 1;
    }

    println!("\n- Completed Processing Pages {} to 1", start_page);
}