use std::fs;
use std::io::Write;

// ============================================================================
// Get last processed page from file
// ============================================================================
pub fn get_last_processed_page_from_file() -> i64 {
    match fs::read_to_string("output/LastPage.txt") {
        Ok(content) => content.trim().parse().unwrap_or(0),
        Err(_) => 0,
    }
}

// ============================================================================
// Save last processed page to file
// ============================================================================
pub fn save_last_page_to_file(page: u64) {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output/LastPage.txt")
        .unwrap();
    writeln!(file, "{}", page).unwrap();
}
