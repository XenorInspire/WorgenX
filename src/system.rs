// Extern crates
use std::fs::File;
#[cfg(feature = "gui")]
use std::io::stdin;
use std::io::Write;
use std::path::Path;

use crate::error::SystemError;

/// OS specific constants for GUI mode
#[cfg(all(target_family = "unix", feature = "gui"))]
pub mod unix {
    pub const HOME_ENV_VAR: &str = "HOME";
    pub const PASSWORDS_FOLDER: &str = "/worgenx/passwords/";
}
#[cfg(all(target_family = "windows", feature = "gui"))]
pub mod windows {
    pub const HOME_ENV_VAR: &str = "USERPROFILE";
    pub const PASSWORDS_FOLDER: &str = "\\worgenx\\passwords\\";
}

/// This function is charged to get user String input y/n
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns an empty String.
///
#[cfg(feature = "gui")]
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
#[cfg(feature = "gui")]
pub fn get_user_choice() -> String {
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => buffer.trim().to_string(),
        Err(_) => String::new(),
    }
}

/// This function is charged to get user int input
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns 0
///
#[cfg(feature = "gui")]
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
/// * `mode` - A string slice that holds the mode of the path/filename to check (FILE or DIRECTORY)
///
/// # Returns
///
/// Ok(String) if the path/filename is valid, containing the full path, SystemError otherwise
///
pub fn is_valid_path(path: String) -> Result<String, SystemError> {
    let filename = match Path::new(&path).file_name() {
        Some(f) => match f.to_str() {
            Some(f) => f.to_string(),
            None => return Err(SystemError::InvalidPath(path.to_string())),
        },
        None => return Err(SystemError::InvalidPath(path.to_string())),
    };

    let invalid_chars: &[char] = get_invalid_chars();
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(SystemError::InvalidFilename(filename.to_string()));
    }

    let full_path = if !Path::new(&path).is_absolute() {
        let current_dir = match std::env::current_dir() {
            Ok(c) => match c.to_str() {
                Some(s) => s.to_string(),
                None => return Err(SystemError::InvalidPath(path.to_string())),
            },
            Err(e) => {
                return Err(SystemError::UnableToCreateFile(
                    path.to_string(),
                    e.to_string(),
                ))
            }
        };
        current_dir + "/" + &filename
    } else {
        path.clone()
    };

    #[cfg(target_family = "windows")]
    if full_path.len() > 260 {
        return Err(SystemError::PathTooLong(path.to_string()));
    }

    if !check_if_folder_exists(&full_path) {
        return Err(SystemError::ParentFolderDoesntExist(path.to_string()));
    }
    Ok(full_path)
}

/// Check if folder exists
///
/// # Arguments
///
/// * `folder` - A string slice that holds the folder to check
///
/// # Returns
///
/// True if the folder exists, false otherwise
///
pub fn check_if_folder_exists(folder: &str) -> bool {
    Path::new(folder).parent().is_some()
}

/// Save the password in a file
///
/// # Arguments
///
/// * `passwords` - A vector of String that holds the passwords to save
///
/// # Returns
///
/// Ok if the password has been saved, SystemError otherwise
///
pub fn save_passwords(file_path: String, passwords: &Vec<String>) -> Result<(), SystemError> {
    let mut file = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(SystemError::UnableToCreateFile(
                file_path.clone(),
                e.to_string(),
            ))
        }
    };

    for password in passwords {
        match file.write_all(password.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                return Err(SystemError::UnableToWriteToFile(
                    file_path.clone(),
                    e.to_string(),
                ))
            }
        }
        match file.write_all(b"\n") {
            Ok(_) => (),
            Err(e) => {
                return Err(SystemError::UnableToWriteToFile(
                    file_path.clone(),
                    e.to_string(),
                ))
            }
        }
    }

    Ok(())
}

/// This function is charged to save the JSON string given in parameter in a file
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path of the file to create
/// * `json_content` - A string slice that holds the JSON string to save
///
/// # Returns
///
/// Ok if the password has been saved, SystemError otherwise
///
#[cfg(feature = "cli")]
pub fn save_json_to_file(file_path: String, json_content: &str) -> Result<(), SystemError> {
    let mut file = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(SystemError::UnableToCreateFile(
                file_path.clone(),
                e.to_string(),
            ))
        }
    };

    match file.write_all(json_content.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            return Err(SystemError::UnableToWriteToFile(
                file_path.clone(),
                e.to_string(),
            ))
        }
    }

    Ok(())
}

/// This function is charged to create the passwords or wordlists folder if it doesn't exist
///
/// # Arguments
///
/// * `folder` - A string slice that holds the folder to create
///
/// # Returns
///
/// A boolean value that indicates if the folder has been created or not
/// Ok if the folder has been created, SystemError otherwise
///
#[cfg(feature = "gui")]
pub fn create_folder_if_not_exists(folder: &str) -> Result<(), SystemError> {
    let mut folder = String::from(folder);
    if folder.pop().is_none() {
        return Err(SystemError::InvalidPath(folder.clone()));
    }
    if !Path::new(&folder).exists() {
        match std::fs::create_dir_all(&folder) {
            Ok(_) => return Ok(()),
            Err(e) => {
                return Err(SystemError::UnableToCreateFolder(
                    folder.clone(),
                    e.to_string(),
                ))
            }
        };
    }
    Ok(())
}

/// This function send the invalid chars for windows path
///
/// # Returns
///
/// '<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n' chars
///
#[cfg(target_family = "windows")]
fn get_invalid_chars() -> &'static [char] {
    &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n',]
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
