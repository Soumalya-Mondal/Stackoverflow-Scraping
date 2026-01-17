#![allow(dead_code)]
use reqwest::Client;

const BASE_URL: &str = "https://stackoverflow.com/questions";
const REQURIED_BLOCK_SELECTOR: &str = "div#questions";
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";
const SINGLE_QUESTION_SELECTOR: &str = "div.bb.bc-black-200";
const QUESTION_NUMBER_SELECTOR: &str = "meta[itemprop='position']";

#[tokio::main]
async fn main() {
    // create web-client object
    let web_client = Client::new();
    // create response object
    let page_response = web_client.get(BASE_URL.to_string())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await
        .unwrap();
    // fetch page content as text
    let whole_page_content: String = page_response
        .text()
        .await
        .unwrap();
    // parse the whole page content
    let whole_html_parse_document = scraper::Html::parse_document(&whole_page_content);
    // select required block
    let required_block_selector = scraper::Selector::parse(REQURIED_BLOCK_SELECTOR)
        .unwrap();
    // get required block element
    let required_block_element = whole_html_parse_document
        .select(&required_block_selector)
        .next()
        .unwrap();
    // get total question count
    let total_question_selector = scraper::Selector::parse(TOTAL_QUESTION_SELECTOR)
        .unwrap();
    // extract total question count
    let total_question_element = required_block_element
        .select(&total_question_selector)
        .next()
        .unwrap();
    // parse total question count
    let total_question_count: usize = total_question_element
        .value()
        .attr("content")
        .unwrap()
        .parse()
        .unwrap();
    println!("Total Questions: {}", total_question_count);
    let total_pages_count: u64 = total_question_count.div_ceil(50) as u64;
    println!("Total Pages: {}", total_pages_count);
}
