// Extern crates
use std::fs::File;
use std::io::{stdin, Write};

const PASSWORD_PATH: &str = "passwords";

// This function is charged to save the random password in a file with \n as separator
pub fn save_passwords_into_a_file(passwords: &Vec<String>) {
    let mut filename = String::new();

    while !is_valid_filename(filename.as_str()) {
        println!("Please enter the file name");
        filename = get_user_choice();
    }

    check_folder_exists(PASSWORD_PATH);
    let mut file = File::create(PASSWORD_PATH.to_string() + "/" + &filename);
    while file.is_err() {
        println!("Unable to create the file '{}': {}", filename, file.unwrap_err());
        println!("Please enter a new file name:");
        filename = get_user_choice();
        file = File::create(&filename);
    }
    
    let mut file = file.unwrap();
    for password in passwords {
        match file.write_all(password.as_bytes()) {
            Ok(_) => (),
            Err(e) => println!("Unable to write data: {}", e),
        }
        match file.write_all(b"\n") {
            Ok(_) => (),
            Err(e) => println!("Unable to write data: {}", e),
        }
    }
}

// This function is charged to get user String input y/n
pub fn get_user_choice_yn() -> String {
    let mut choice = get_user_choice();
    while !choice.eq("y") && !choice.eq("n") {
        println!("Please enter a valid answer (y/n)");
        choice = get_user_choice();
    }

    choice
}

// This function is charged to get user String input
pub fn get_user_choice() -> String {
    let mut buffer = String::new();
    let result = stdin().read_line(&mut buffer);
    match result {
        Ok(_) => buffer.trim().to_string(),
        Err(e) => {
            println!("Error: {}", e);
            String::new()
        }
    }
}

// This function is charged to get user int input
pub fn get_user_choice_int() -> u64 {
    let mut is_good_number = false;
    let mut number: u64 = 0;

    while !is_good_number {
        let choice = get_user_choice();
        match choice.trim().parse::<u64>() {
            Ok(_n) => {
                if _n > 0 {
                    is_good_number = true;
                    number = _n;
                } else {
                    println!("Please enter a valid number greater than 0");
                }
            }
            Err(_e) => println!("Please enter a valid number greater than 0, {}", _e),
        }
    }

    number
}

// This function is charged to check a filename
pub fn is_valid_filename(filename: &str) -> bool {
    if filename.is_empty() { return false };

    // For Windows platforms
    #[cfg(windows)]
    const INVALID_CHARS: &[char] = &[
        '<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@',
    ];
    // For other platforms
    #[cfg(not(windows))]
    const INVALID_CHARS: &[char] = &['/', '\0'];

    !filename.chars().any(|c| INVALID_CHARS.contains(&c))
}

// Check if folder exists unless create it
pub fn check_folder_exists(folder: &str) {
    if !std::path::Path::new(folder).exists() {
        std::fs::create_dir(folder).unwrap();
    }
}
