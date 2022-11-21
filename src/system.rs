// Extern crates
use std::fs::File;
use std::io::{stdin, Write};

// This function is charged to save the password in a file with \n as separator
pub fn save_into_a_file(passwords: &Vec<String>, file_name: &str) {
    let mut file = File::create(file_name).expect("Unable nto create file");
    for password in passwords {
        file.write_all(password.as_bytes()).expect("Unable to write data");
        file.write_all(b"\n").expect("Unable to write data");
    }
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
    let mut is_good_number = false;
    let mut number: u64 = 0;

    while !is_good_number {
        let mut buffer = String::new();
        stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line from stdin");
        buffer = buffer.replace("\n", "");
        match buffer.trim().parse::<u64>() {
            Ok(_n) => {
                println!("You entered {}", _n);
                if _n > 0 {
                    is_good_number = true;
                    number = _n;
                }
            }
            Err(_e) => println!("Please enter a valid number greater than 0, {}", _e),
        }
    }

    number
}
