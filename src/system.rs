// Extern crates
use std::fs::File;
use std::io::{stdin, Write};
use std::path::Path;

use crate::error::SystemError;

/// Theses constants are charged to store the path of the wordlists and passwords folders
pub const PASSWORD_PATH: &str = "passwords";
const WORDLIST_PATH: &str = "wordlists";

/// This function is charged to get user String input y/n
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns an empty String.
///
pub fn get_user_choice_yn() -> String {
    let mut choice = get_user_choice();
    while !choice.eq("y") && !choice.eq("n") {
        println!("Please enter a valid answer (y/n)");
        choice = get_user_choice();
    }

    choice
}

/// This function is charged to get user String input
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns an empty String.
///
/// # Example
/// ```
/// let choice = system::get_user_choice();
/// ```
///
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

/// This function is charged to get user int input
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns 0.
///
pub fn get_user_choice_int() -> u64 {
    let mut is_good_number = false;
    let mut number: u64 = 0;

    while !is_good_number {
        let choice = get_user_choice();
        if choice.is_empty() {
            println!("Please enter a valid number greater than 0");
            continue;
        }
        match choice.trim().parse::<u64>() {
            Ok(n) => {
                if n > 0 {
                    is_good_number = true;
                    number = n;
                } else {
                    println!("Please enter a valid number greater than 0");
                }
            }
            Err(_e) => println!("Please enter a valid number greater than 0, {}", _e),
        }
    }

    number
}

/// This function is charged to check a path/filename
///
/// # Arguments
///
/// * `path` - A string slice that holds the path/filename to check
///
/// # Returns
///
/// A boolean value that indicates if the path is valid or not
///
pub fn is_valid_path(path: &str) -> Result<(), SystemError> {
    let invalid_chars: &[char] = get_invalid_chars();

    #[cfg(target_family = "windows")]
    if path.len() > 260 {
        return Err(SystemError::PathTooLong(path.to_string()));
    }

    if !check_if_folder_exists(path) {
        return Err(SystemError::ParentFolderDoesntExist(path.to_string()));
    }

    if path.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(SystemError::InvalidPath(path.to_string()));
    } else {
        return Ok(());
    }
}

/// Check if folder exists
///
/// # Arguments
///
/// * `folder` - A string slice that holds the folder to check
///
/// # Returns
///
/// A boolean value that indicates if the folder exists or not
/// True if the folder exists, false otherwise
///
pub fn check_if_folder_exists(folder: &str) -> bool {
    match Path::new(folder).parent() {
        Some(_) => return true,
        None => {
            return false;
        }
    };
}

/// This function send the invalid chars for windows path
///
/// # Returns
///
/// '<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n' chars
///
#[cfg(target_family = "windows")]
fn get_invalid_chars() -> &'static [char] {
    &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n']
}

/// This function send the invalid chars for unix platforms path
///
/// # Returns
///
/// '/', '\0', '\r', '\n' chars
///
#[cfg(target_family = "unix")]
fn get_invalid_chars() -> &'static [char] {
    &['/', '\0', '\r', '\n']
}
