mod errors;
mod lexical;
mod parsing;
use std::fs;

fn time_test(test: String, file_size_bytes: usize, process: impl Fn()) {
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

fn get_file_contents(file_name: &str) -> (String, usize) {
    let file = fs::File::open(file_name).unwrap();
    let file_size_bytes = file.metadata().unwrap().len().try_into().unwrap();

    let contents = fs::read_to_string(file_name).expect("Should have been able to read the file");

    (contents, file_size_bytes)
}

fn read_json(file_name: &str) {
    // let (contents, file_size_bytes) = get_file_contents(file_name);
    let contents = fs::read_to_string(file_name).expect("Should have been able to read the file");
    let file_size_bytes = contents.len();
    time_test(
        format!("read {file_name}"),
        file_size_bytes,
        || match parsing::Parser::parse(&contents) {
            Ok(json) => {
                // println!("{json:#?}");
            }
            Err(error) => panic!("error: {:?}", error[0]),
        },
    );
}

fn main() {
    read_json("tests/canada.json");
    read_json("tests/twitter.json");
}
