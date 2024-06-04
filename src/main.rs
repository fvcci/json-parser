mod errors;
mod lexical;
mod parser;
use std::fs;

fn time_test(test: &str, file_size_bytes: u64, process: impl Fn()) {
    const NUM_RUNS: u32 = 100;

    let start_time = std::time::Instant::now();
    for _ in 0..NUM_RUNS {
        process();
    }
    let end_time = std::time::Instant::now();

    let mbs = file_size_bytes as f64 / 1_000_000.0;
    let mbps = mbs * NUM_RUNS as f64 / ((end_time - start_time).as_secs_f64());

    println!("[{test}] Parsing speed: {mbps:.2} MB/s");
}

fn get_file_contents(file_name: &str) -> (String, u64) {
    let file = fs::File::open(file_name).unwrap();
    let file_size_bytes = file.metadata().unwrap().len();

    let contents = fs::read_to_string(file_name).expect("Should have been able to read the file");

    (contents, file_size_bytes)
}

fn read_canada_json() {
    let (contents, file_size_bytes) = get_file_contents("tests/canada.json");
    let contents_str = contents.as_str();
    time_test(
        "read_canada_json",
        file_size_bytes,
        || match parser::parse(contents_str) {
            Ok(json) => {
                // println!("{json:#?}");
            }
            Err(error) => panic!("error: {:?}", error[0]),
        },
    );
}

fn read_twitter_json() {
    let (contents, file_size_bytes) = get_file_contents("tests/twitter.json");
    let contents_str = contents.as_str();
    time_test(
        "read_twitter_json",
        file_size_bytes,
        || match parser::parse(contents_str) {
            Ok(json) => {
                // println!("{json:#?}");
            }
            Err(error) => panic!("error: {:?}", error[0]),
        },
    );
}

fn main() {
    read_canada_json();
    read_twitter_json();
}
