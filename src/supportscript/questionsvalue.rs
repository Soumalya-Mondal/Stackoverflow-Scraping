use scraper::{Html, Selector};
use chrono::{Local, Datelike, Timelike};

// ============================================================================
// Constants
// ============================================================================
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";
const QUESTION_BLOCK_SELECTOR: &str = "div.s-post-summary.js-post-summary";
const TITLE_SELECTOR: &str = "h3.s-post-summary--content-title a span[itemprop='name']";
const LINK_SELECTOR: &str = "h3.s-post-summary--content-title a.s-link";
const TIMESTAMP_SELECTOR: &str = "time.s-user-card--time span.relativetime";

// ============================================================================
// Data Structures
// ============================================================================
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

// ============================================================================
// Parse Questions Function
// ============================================================================
pub fn parse_questions(page_html: &str) -> Vec<QuestionRow> {
    let doc = Html::parse_document(page_html);

    let required_block_sel = Selector::parse(REQUIRED_BLOCK_SELECTOR).unwrap();
    let question_block_sel = Selector::parse(QUESTION_BLOCK_SELECTOR).unwrap();
    let title_sel = Selector::parse(TITLE_SELECTOR).unwrap();
    let link_sel = Selector::parse(LINK_SELECTOR).unwrap();
    let timestamp_sel = Selector::parse(TIMESTAMP_SELECTOR).unwrap();

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

        let timestamp_str = q
            .select(&timestamp_sel)
            .next()
            .and_then(|n| n.value().attr("title"))
            .map(|s| s.trim_end_matches('Z').to_string())
            .unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

        // Parse timestamp string into components
        let (q_year, q_month, q_day, q_hour, q_min, q_sec) = parse_timestamp(&timestamp_str);

        results.push(QuestionRow { title, id, q_year, q_month, q_day, q_hour, q_min, q_sec });
    }

    results
}

// ============================================================================
// Parse Timestamp Helper Function
// ============================================================================
fn parse_timestamp(timestamp: &str) -> (u16, u8, u8, u8, u8, u8) {
    // Expected format: "2026-01-23 13:32:20"
    let parts: Vec<&str> = timestamp.split_whitespace().collect();
    if parts.len() != 2 {
        let now = Local::now();
        return (
            now.year() as u16,
            now.month() as u8,
            now.day() as u8,
            now.hour() as u8,
            now.minute() as u8,
            now.second() as u8,
        );
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    let year = date_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let month = date_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let day = date_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let hour = time_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let min = time_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let sec = time_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    (year, month, day, hour, min, sec)
}