use std::fs::File;
use std::io::{self, BufRead};

fn read_file_lines(file_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut lines = Vec::new();

    for line in reader.lines() {
        lines.push(line?);
    }

    Ok(lines)
}

fn main() -> io::Result<()> {
    let file_path_mono_threaded = "../with_threads/passwords.txt";
    let file_path_multi_threaded = "../without_threads/passwords.txt";

    let lines_mono_threaded = read_file_lines(file_path_mono_threaded)?;
    let lines_multi_threaded = read_file_lines(file_path_multi_threaded)?;

    let mut sorted_lines_mono_threaded = lines_mono_threaded.clone();
    let mut sorted_lines_multi_threaded = lines_multi_threaded.clone();
    sorted_lines_mono_threaded.sort();
    sorted_lines_multi_threaded.sort();

    let equal = sorted_lines_mono_threaded == sorted_lines_multi_threaded;

    if equal {
        println!("Same lines in both files.");
    } else {
        println!("Different lines in both files.");
    }

    Ok(())
}
