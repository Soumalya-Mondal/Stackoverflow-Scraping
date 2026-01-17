use reqwest::Client;

const BASE_URL: &str = "https://stackoverflow.com/questions";
const ALL_QUESTION_SELECTOR: &str = "div#questions";
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
    let whole_html_parse_document = scraper::Html::parse_document(&whole_page_content);
    // let all_question_selector = scraper::Selector::parse(ALL_QUESTION_SELECTOR)
    //     .unwrap();
    // let all_question_elements = whole_html_parse_document
    //     .select(&all_question_selector)
    //     .next()
    //     .unwrap();
    // let per_question_selector = scraper::Selector::parse(SINGLE_QUESTION_SELECTOR)
    //     .unwrap();
    // let per_question_elements = all_question_elements.select(&per_question_selector);
    // let question_number_selector = scraper::Selector::parse(QUESTION_NUMBER_SELECTOR)
    //     .unwrap();
    // for single_question_element in per_question_elements {
    //     if let Some(meta_element) = single_question_element.select(&question_number_selector).next()
    //         && let Some(content) = meta_element.value().attr("content") {
    //             println!("{}", content);
    //         }
    // }
}
