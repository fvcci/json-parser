mod lexical;
mod parser;
use std::fs;

fn time_test(test: &str, bytes_to_parse: usize, process: impl Fn()) {
    let start_time = std::time::Instant::now();
    process();
    let end_time = std::time::Instant::now();

    let mbs = bytes_to_parse as f64 / 1_000_000.0;
    let mbps = mbs / (end_time - start_time).as_secs_f64();

    println!("[{test}] Parsing speed: {mbps:.2} MB/s");
}

fn read_canada_json() {
    let contents =
        fs::read_to_string("tests/canada.json").expect("Should have been able to read the file");
    time_test(
        "read_canada_json",
        contents.len() * 1000,
        || match parser::parse(contents.as_str()) {
            Ok(_) => {}
            Err(error) => panic!("error: {:?}", error[0]),
        },
    );
}

fn read_twitter_json() {
    let contents =
        fs::read_to_string("tests/twitter.json").expect("Should have been able to read the file");
    time_test(
        "read_twitter_json",
        contents.len() * 1000,
        || match parser::parse(contents.as_str()) {
            Ok(_) => {}
            Err(error) => panic!("error: {:?}", error[0]),
        },
    );
}

fn main() {
    read_canada_json();
    read_twitter_json();
}
