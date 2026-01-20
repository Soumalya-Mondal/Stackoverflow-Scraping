# Stack Overflow Scraper

A simple yet robust web scraper built in Rust to extract question titles and their corresponding IDs from Stack Overflow. It's designed to be efficient and resilient, with features to handle interruptions and scrape politely.

## Features

- **Efficient Scraping**: Uses `reqwest` for asynchronous HTTP requests and `scraper` for fast HTML parsing.
- **Resume Capability**: Automatically resumes scraping from the last successfully processed page, making it resilient to interruptions.
- **Polite Scraping**: Implements random delays between requests to avoid overloading Stack Overflow's servers.
- **Organized Output**: Saves scraped data into individual CSV files for each page in the `output/questions-id/` directory.
- **Error Handling**: Logs pages that fail to fetch for later review and retries.
- **Optimized for Performance**: The release profile is configured for high performance with LTO and other optimizations.

## How It Works

The scraper operates by starting from the most recent page of questions on Stack Overflow and working its way backward.

1.  **Initialization**:
    - Creates necessary `output` directories.
    - Reads `output/LastPage.txt` to determine where to resume. If the file doesn't exist, it starts from the very last page.

2.  **Metadata Fetch**:
    - It makes an initial request to Stack Overflow to get the total number of questions.
    - From this, it calculates the total number of pages.

3.  **Scraping Loop**:
    - It processes pages in reverse chronological order (from newest to oldest).
    - It scrapes a maximum of `500` pages per run to keep executions short and manageable.
    - For each page:
        - A random delay (0.1 to 1.9 seconds) is introduced.
        - The page HTML is fetched.
        - Question titles and IDs are parsed from the HTML.
        - The data is written to a corresponding `{page_number}.csv` file.
        - `LastPage.txt` is updated with the page number that was just processed.

4.  **Completion**:
    - Once the run is complete, it prints a summary of the pages processed.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Cargo (comes with Rust)

## Getting Started

### 1. Clone the repository

```sh
git clone https://github.com/Soumalya-Mondal/Stackoverflow-Scraping.git
cd Stackoverflow-Scraping
```

### 2. Build the project

For a development build:
```sh
cargo build
```

For an optimized release build (recommended for running):
```sh
cargo build --release
```

## Usage

To run the scraper, execute the following command from the project root:

```sh
cargo run --release
```

The scraper will start, create the `output` directory if it doesn't exist, and begin processing pages. You can run it multiple times, and it will pick up where it left off.

## Output Structure

The scraper generates the following files and directories:

- `output/`: The main directory for all generated files.
- `output/questions-id/`: Contains the CSV files with scraped data.
    - `12345.csv`: An example CSV file, where `12345` is the page number. Each file contains "Question" and "ID" columns.
- `output/LastPage.txt`: A text file containing the page number of the last page that was successfully scraped. This is used for resuming.
- `output/LostPage.txt`: A log of page numbers that could not be fetched due to network errors or non-success HTTP status codes.
