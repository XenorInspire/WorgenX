// Extern crates
use std::fs::File;
use std::io::{stdin, Write};

// This function is charged to save the password in a file
fn save_into_a_file(password: &str, file_name: &str) {
    let mut file = File::create(file_name).expect("Unable to create file");
    file.write_all(password.as_bytes())
        .expect("Unable to write data");
}

// This function is charged to get user String input
pub fn get_user_choice() -> String {
    let mut buffer = String::new();
    stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line from stdin");
    buffer.trim().to_string()
}

// This function is charged to get user int input
pub fn get_user_choice_int() -> u64 {
    let mut buffer = String::new();
    let mut is_good_number = false;
    let mut number: u64 = 0;

    while !is_good_number {
        stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line from stdin");
        match buffer.trim().parse::<u64>() {
            Ok(_n) => {
                if _n > 0 {
                    is_good_number = true;
                    number = _n;
                }
            }
            Err(_e) => println!("Please enter a valid number greater than 0"),
        }
    }

    number
}
