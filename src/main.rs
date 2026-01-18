use reqwest::Client;
use rand::Rng;
use tokio::time::{sleep, Duration};
use scraper::{Html, Selector};
use std::fs;
use csv::Writer;
use std::io::Write;

const BASE_URL: &str = "https://stackoverflow.com/questions";
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
const QUESTION_BLOCK_SELECTOR: &str = "div.s-post-summary.js-post-summary";
const TITLE_SELECTOR: &str = "h3.s-post-summary--content-title a span[itemprop='name']";
const LINK_SELECTOR: &str = "h3.s-post-summary--content-title a.s-link";
const STAT_ITEM_SELECTOR: &str = "div.s-post-summary--stats-item";
const STAT_NUM_SELECTOR: &str = "span.s-post-summary--stats-item-number";
const STAT_UNIT_SELECTOR: &str = "span.s-post-summary--stats-item-unit";

#[derive(Debug, Clone)]
struct QuestionRow {
    title: String,
    views: u64,
    id: u64,
}

fn parse_questions_views_and_links(page_html: &str) -> Vec<QuestionRow> {
    let doc = Html::parse_document(page_html);

    let required_block_sel = Selector::parse(REQUIRED_BLOCK_SELECTOR).unwrap();
    let question_block_sel = Selector::parse(QUESTION_BLOCK_SELECTOR).unwrap();
    let title_sel = Selector::parse(TITLE_SELECTOR).unwrap();
    let link_sel = Selector::parse(LINK_SELECTOR).unwrap();

    let stat_item_sel = Selector::parse(STAT_ITEM_SELECTOR).unwrap();
    let stat_num_sel = Selector::parse(STAT_NUM_SELECTOR).unwrap();
    let stat_unit_sel = Selector::parse(STAT_UNIT_SELECTOR).unwrap();

    let mut results = Vec::new();

    let required_block = match doc.select(&required_block_sel).next() {
        Some(b) => b,
        None => return results,
    };

    for q in required_block.select(&question_block_sel) {
        // ---- Title ----
        let title = q
            .select(&title_sel)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.is_empty() {
            continue;
        }

        // ---- Link (href) ----
        // Usually relative: "/questions/...."
        let href = q
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("")
            .to_string();

        // Extract ID from href (e.g., "/questions/79870378/..." -> 79870378)
        let id: u64 = href
            .split('/')
            .nth(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // ---- Views ----
        let mut views: u64 = 0;
        for item in q.select(&stat_item_sel) {
            let unit = item
                .select(&stat_unit_sel)
                .next()
                .map(|u| u.text().collect::<String>().trim().to_lowercase())
                .unwrap_or_default();

            if unit == "views" {
                let num_txt = item
                    .select(&stat_num_sel)
                    .next()
                    .map(|n| n.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let cleaned = num_txt.replace(',', "");
                if let Ok(v) = cleaned.parse::<u64>() {
                    views = v;
                }
                break;
            }
        }

        results.push(QuestionRow { title, views, id });
    }

    results
}

#[tokio::main]
async fn main() {
    let web_client = Client::new();

    // Create output directory if it doesn't exist
    fs::create_dir_all("output").unwrap();

    // Initialize CSV writer with UTF-8 BOM
    let mut file = fs::File::create("output/StackOverFlowQuestions.csv").unwrap();
    file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap(); // UTF-8 BOM
    let mut writer = Writer::from_writer(file);
    writer.write_record(["Title", "Views", "ID"]).unwrap();

    // Fetch first page to compute total pages
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

    for page in 1..=total_pages_count {
        let url: String = format!("{}?page={}&pagesize=50", BASE_URL, page);

        sleep(Duration::from_secs_f64(rand::rng().random_range(1.0..2.0))).await;

        let resp = match web_client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to fetch page {}: {}", page, e);
                continue;
            }
        };

        if !resp.status().is_success() {
            eprintln!("Failed page {}: status {}", page, resp.status());
            continue;
        }

        println!("Page {}: status {}", page, resp.status());

        let html = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to read HTML page {}: {}", page, e);
                continue;
            }
        };

        let items = parse_questions_views_and_links(&html);
        if items.is_empty() {
            continue;
        }

        for item in items {
            writer.write_record(&[
                item.title,
                item.views.to_string(),
                item.id.to_string(),
            ]).unwrap();
        }
    }

    writer.flush().unwrap();
}
