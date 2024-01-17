// Internal crates
use crate::error::{SystemError, WorgenXError};

// Extern crates
#[cfg(feature = "gui")]
use std::io::stdin;
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

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

/// This function is charged to create the passwords or wordlists folder if it doesn't exist
///
/// # Arguments
///
/// * `folder` - A string slice that holds the folder to create
///
/// # Returns
///
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

/// This function is charged to calculate the elapsed time between two timestamps
/// The result is returned in human readable format (hours, minutes, seconds depending on the elapsed time)
///
/// # Arguments
///
/// * `start_time` - The start timestamp
///
/// # Returns
///
/// A string slice containing the elapsed time in human readable format
///
pub fn get_elapsed_time(start_time: Instant) -> String {
    let elapsed_time = start_time.elapsed();
    let mut elapsed_time = elapsed_time.as_secs();
    let mut elapsed_time_str = String::new();

    if elapsed_time >= 3600 {
        let hours = elapsed_time / 3600;
        elapsed_time -= hours * 3600;
        elapsed_time_str.push_str(&hours.to_string());
        elapsed_time_str.push_str(" hours and ");
    }
    if elapsed_time >= 60 {
        let minutes = elapsed_time / 60;
        elapsed_time -= minutes * 60;
        elapsed_time_str.push_str(&minutes.to_string());
        elapsed_time_str.push_str("minutes and ");
    }
    elapsed_time_str.push_str(&elapsed_time.to_string());
    elapsed_time_str.push_str(" seconds");

    elapsed_time_str
}

/// This function is charged to save the generated passwords in a file and send the progress/possible errors to the channel
///
/// # Arguments
///
/// * `file` - The file to write to, wrapped in an Arc<Mutex<File>>
/// * `passwords` - The passwords to write
///
pub fn save_passwd_to_file(file: Arc<Mutex<File>>, passwords: String) -> Result<(), WorgenXError> {
    let mut file = match file.lock() {
        Ok(file) => file,
        Err(_) => {
            return Err(WorgenXError::SystemError(SystemError::UnableToWriteToFile(
                "output file".to_string(),
                "Please check the path, the permissions and try again".to_string(),
            )))
        }
    };
    match file.write_all(format!("{}\n", passwords).as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(WorgenXError::SystemError(SystemError::UnableToWriteToFile(
            "output file".to_string(),
            "Please check the path, the permissions and try again".to_string(),
        ))),
    }
}
