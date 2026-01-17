use reqwest::Client;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::time::Instant;

const BASE_URL: &str = "https://stackoverflow.com/questions";
const REQURIED_BLOCK_SELECTOR: &str = "div#questions";
const TOTAL_QUESTION_SELECTOR: &str = "meta[itemprop='numberOfItems']";

#[tokio::main]
async fn main() {
    // create output directory
    create_dir_all("output").unwrap();

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

    // find total pages count
    let total_pages_count: u64 = total_question_count.div_ceil(50) as u64;

    // loop through all pages and store HTML content
    for page in (1..=total_pages_count).rev() {
        let start = Instant::now();
        let url: String = format!("{}?page={}&pagesize=50", BASE_URL, page);

        // fetch page HTML content
        let page_response = web_client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send()
            .await
            .unwrap();

        // convert response to text
        let page_html_content: String = page_response
            .text()
            .await
            .unwrap();

        // create and write to page file
        let page_file_path = format!("output/{}.txt", page);
        let mut page_file = File::create(page_file_path)
            .unwrap();
        page_file.write_all(page_html_content.as_bytes())
            .unwrap();

        let elapsed = start.elapsed();
        println!("Saved page {} in {:.2} seconds", page, elapsed.as_secs_f64());
    }
}
