use scraper::{Html, Selector};

// ============================================================================
// Constants
// ============================================================================
const REQUIRED_BLOCK_SELECTOR: &str = "div#questions";
const QUESTION_BLOCK_SELECTOR: &str = "div.s-post-summary.js-post-summary";
const TITLE_SELECTOR: &str = "h3.s-post-summary--content-title a span[itemprop='name']";
const LINK_SELECTOR: &str = "h3.s-post-summary--content-title a.s-link";

// ============================================================================
// Data Structures
// ============================================================================
#[derive(Debug, Clone)]
pub struct QuestionRow {
    pub title: String,
    pub id: i64,
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
