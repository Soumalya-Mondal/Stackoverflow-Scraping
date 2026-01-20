// ============================================================================
// Importing External Crates
// ============================================================================
use reqwest::Client;
use rand::Rng;
use tokio::time::{sleep, Duration};
use scraper::{Html, Selector};
use std::fs;
use csv::Writer;
use std::io::Write;
use std::io::Read;

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
const PAGES_PER_RUN: u64 = 500;

// ============================================================================
// Data Structures
// ============================================================================
#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    id: u64,
}

// ============================================================================
// Read Last Page From File Function
// ============================================================================
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

// ============================================================================
// Write Last Page To File Function
// ============================================================================
fn write_last_page(page: u64) {
    if let Ok(mut file) = fs::File::create("output/LastPage.txt") {
        let _ = writeln!(file, "{}", page);
    }
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

        let id: u64 = href
            .split('/')
            .nth(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        results.push(QuestionRow { title, id });
    }

    results
}

// ============================================================================
// Main Async Function
// ============================================================================
#[tokio::main]
async fn main() {
    let web_client = Client::new();

    fs::create_dir_all("output")
        .unwrap();

    fs::create_dir_all("output/questions-id")
        .unwrap();

    let last_processed_page = read_last_page();
    println!("- Last Processed Page: {}", last_processed_page);

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

    let starting_page = if last_processed_page == 0 {
        total_pages_count
    } else if last_processed_page == 1 {
        println!("- All Pages Have Been Processed!");
        return;
    } else {
        last_processed_page - 1
    };

    let ending_page = if starting_page > PAGES_PER_RUN {
        starting_page - PAGES_PER_RUN + 1
    } else {
        1
    };

    println!("- Processing Pages {} To {} [{}-Pages]\n", starting_page, ending_page, PAGES_PER_RUN);

    let mut page_count: u64 = 1;
    for page in (ending_page..=starting_page).rev() {
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
                eprintln!("{} - Failed To Fetch Page {}: {}", page_count, page, e);
                continue;
            }
        };

        println!("- [{:03}/{}] Page: {}; Response: {}", page_count, PAGES_PER_RUN, page, resp.status());

        if !resp.status().is_success() {
            eprintln!("{} - Failed Page {}: Status {}", page_count, page, resp.status());

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
                eprintln!("{} - Failed To Read HTML Page {}: {}", page_count, page, e);
                continue;
            }
        };

        let items = parse_questions_views_and_links(&html);
        if items.is_empty() {
            continue;
        }

        let csv_path: String = format!("output/questions-id/{}.csv", page);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&csv_path)
            .unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        let mut writer = Writer::from_writer(file);

        writer.write_record(["Question", "ID"]).unwrap();

        for item in items {
            writer.write_record(&[
                item.title,
                item.id.to_string(),
            ]).unwrap();
        }

        writer.flush().unwrap();

        write_last_page(page);

        page_count += 1;
    }

    println!("\n- Completed Processing {} Pages. Last Page Processed: {}", page_count - 1, ending_page);
}