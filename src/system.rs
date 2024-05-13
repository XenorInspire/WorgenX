// Internal crates
use crate::error::{SystemError, WorgenXError};

// Extern crates
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "gui")]
use std::{cmp::PartialOrd, default::Default, fmt::Display, io::stdin, marker::Copy, str::FromStr};
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// OS specific constants for GUI mode.
///
/// * `HOME_ENV_VAR` - The environment variable that holds the user's home directory.
/// * `PASSWORDS_FOLDER` - The folder where the passwords will be saved.
/// * `WORDLISTS_FOLDER` - The folder where the wordlists will be saved.
///
#[cfg(all(target_family = "unix", feature = "gui"))]
pub mod unix {
    pub const HOME_ENV_VAR: &str = "HOME";
    pub const PASSWORDS_FOLDER: &str = "/worgenx/passwords/";
    pub const WORDLISTS_FOLDER: &str = "/worgenx/wordlists/";
}
#[cfg(all(target_family = "windows", feature = "gui"))]
pub mod windows {
    pub const HOME_ENV_VAR: &str = "USERPROFILE";
    pub const PASSWORDS_FOLDER: &str = "\\worgenx\\passwords\\";
    pub const WORDLISTS_FOLDER: &str = "\\worgenx\\wordlists\\";
}

/// This function is charged to get user String input y/n.
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns an empty String.
///
#[cfg(feature = "gui")]
pub fn get_user_choice_yn() -> String {
    let mut choice: String = get_user_choice();
    while !choice.eq("y") && !choice.eq("n") {
        println!("Please enter a valid answer (y/n)");
        choice = get_user_choice();
    }

    choice
}

/// This function is charged to get user String input.
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns an empty String.
///
#[cfg(feature = "gui")]
pub fn get_user_choice() -> String {
    let mut buffer: String = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_) => buffer.trim().to_string(),
        Err(_) => String::new(),
    }
}

/// This function is charged to get user int input.
/// The function will keep asking the user to enter a valid number greater than 0 until the user does so.
/// This is a generic function, so it can be used for any basic integer type.
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns 0.
///
#[cfg(feature = "gui")]
pub fn get_user_choice_int<T>() -> T
where
    T: FromStr + Display + PartialOrd + Copy + Default,
    <T as FromStr>::Err: Display,
{
    let mut is_good_number: bool = false;
    let mut number: T = T::default();

    while !is_good_number {
        let choice: String = get_user_choice();
        if choice.is_empty() {
            println!("Please enter a valid number greater than 0");
            continue;
        }
        match choice.trim().parse::<T>() {
            Ok(n) => {
                if n > T::default() {
                    is_good_number = true;
                    number = n;
                } else {
                    println!("Please enter a valid number greater than 0");
                }
            }
            Err(e) => println!("Please enter a valid number greater than 0, {}", e),
        }
    }

    number
}

/// This function is charged to check a path/filename.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path/filename to check.
///
/// # Returns
///
/// Ok(String) if the path/filename is valid, containing the full path, SystemError otherwise.
///
pub fn is_valid_path(path: String) -> Result<String, SystemError> {
    let filename: String = match Path::new(&path).file_name() {
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

    let full_path: String = if !Path::new(&path).is_absolute() {
        let current_dir: String = match std::env::current_dir() {
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
        current_dir + "/" + &path
    } else {
        path.clone()
    };

    #[cfg(target_family = "windows")]
    if full_path.len() > 260 {
        return Err(SystemError::PathTooLong(path.to_string()));
    }

    if !check_if_parent_folder_exists(&full_path) {
        return Err(SystemError::ParentFolderDoesntExist(path.to_string()));
    }
    Ok(full_path)
}

/// This function is charged to check if the parent folder exists from a given file path.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the file path.
///
/// # Returns
///
/// True if the parent folder exists, false otherwise.
///
pub fn check_if_parent_folder_exists(file_path: &str) -> bool {
    match Path::new(file_path).parent() {
        Some(p) => p.exists(),
        None => false,
    }
}

/// This function is charged to create the passwords or wordlists folder if it doesn't exist.
///
/// # Arguments
///
/// * `folder` - A string slice that holds the folder to create.
///
/// # Returns
///
/// Ok if the folder has been created, SystemError otherwise.
///
#[cfg(feature = "gui")]
pub fn create_folder_if_not_exists(folder: &str) -> Result<(), SystemError> {
    let mut folder: String = String::from(folder);
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

/// This function send the invalid chars for windows path.
///
/// # Returns
///
/// '<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n' chars.
///
#[cfg(target_family = "windows")]
fn get_invalid_chars() -> &'static [char] {
    &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n',]
}

/// This function send the invalid chars for unix platforms path.
///
/// # Returns
///
/// '/', '\0', '\r', '\n' chars.
///
#[cfg(target_family = "unix")]
fn get_invalid_chars() -> &'static [char] {
    &['/', '\0', '\r', '\n']
}

/// This function is charged to calculate the elapsed time between two timestamps.
/// The result is returned in human readable format (hours, minutes, seconds depending on the elapsed time).
///
/// # Arguments
///
/// * `start_time` - The start timestamp.
///
/// # Returns
///
/// A string slice containing the elapsed time in human readable format.
///
pub fn get_elapsed_time(start_time: Instant) -> String {
    let elapsed_time: Duration = start_time.elapsed();
    let mut elapsed_time: u64 = elapsed_time.as_secs();
    let mut elapsed_time_str: String = String::new();

    if elapsed_time >= 3600 {
        let hours: u64 = elapsed_time / 3600;
        elapsed_time -= hours * 3600;
        elapsed_time_str.push_str(&hours.to_string());
        elapsed_time_str.push_str(" hour(s) and ");
    }
    if elapsed_time >= 60 {
        let minutes: u64 = elapsed_time / 60;
        elapsed_time -= minutes * 60;
        elapsed_time_str.push_str(&minutes.to_string());
        elapsed_time_str.push_str("minute(s) and ");
    }

    if elapsed_time == 0 {
        elapsed_time_str.push_str("less than a second");
    } else {
        elapsed_time_str.push_str(&elapsed_time.to_string());
        elapsed_time_str.push_str(" seconds");
    }

    elapsed_time_str
}

/// This function is charged to save the generated passwords in a file and send the progress/possible errors to the channel.
///
/// # Arguments
///
/// * `file` - The file to write to, wrapped in an `Arc<Mutex<File>>`.
/// * `passwords` - The passwords to write.
///
/// # Returns
///
/// Ok if the passwords have been written to the file, WorgenXError otherwise.
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

/// This function is charged to return the progress used by the program.
///
/// # Returns
///
/// A ProgressBar from the indicatif crate.
///
pub fn get_progress_bar() -> indicatif::ProgressBar {
    let pb: ProgressBar = indicatif::ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.green}] {pos:>7}% | {msg}")
            .unwrap_or(ProgressStyle::default_bar()) // Provide the default argument
            .progress_chars("##-"),
    );
    pb
}

/// This functions is charged to return the estimated size of the wordlist.
///
/// # Arguments
///
/// * `nb_of_passwords` - The number of passwords in the wordlist.
/// * `length` - The length of the passwords.
///
/// # Returns
///
/// The estimated size of the wordlist in human readable format.
/// It sends the size in bytes, kilobytes, megabytes, gigabytes, terabytes depending on the size.
/// If the size is less than 1KB, it will return the size in bytes.
/// The function will return an empty string if the parameters are equal to 0.
///
pub fn get_estimated_size(nb_of_passwords: u64, length: u64) -> String {
    if nb_of_passwords == 0 || length == 0 {
        return String::new();
    }

    let size: u64 = nb_of_passwords * (length + 1); // +1 for the newline character
    let mut size_str: String = String::new();
    if size < 1024 {
        size_str.push_str(&size.to_string());
        size_str.push_str(" bytes");
    } else if size < 1048576 {
        size_str.push_str(&format!("{:.2}", size as f64 / 1024.0));
        size_str.push_str(" KB");
    } else if size < 1073741824 {
        size_str.push_str(&format!("{:.2}", size as f64 / 1048576.0));
        size_str.push_str(" MB");
    } else if size < 1099511627776 {
        size_str.push_str(&format!("{:.2}", size as f64 / 1073741824.0));
        size_str.push_str(" GB");
    } else {
        size_str.push_str(&format!("{:.2}", size as f64 / 1099511627776.0));
        size_str.push_str(" TB");
    }
    size_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_path() {
        let relative_path: String = "./test.txt".to_string();
        let invalid_path: String = "test.txt\0".to_string();

        #[cfg(target_family = "windows")]
        let absolute_path: String = "C:/Users/test.txt".to_string();

        #[cfg(target_family = "unix")]
        let absolute_path: String = "/home/test.txt".to_string();

        assert!(is_valid_path(relative_path.clone()).is_ok());
        assert!(is_valid_path(absolute_path.clone()).is_ok());
        assert!(is_valid_path(invalid_path.clone()).is_err());
    }

    #[test]
    fn test_check_if_folder_exists() {
        let valid_relative_file_path: &str = "./test.txt";
        let invalid_file_path: &str = "./test/test.txt";

        #[cfg(target_family = "windows")]
        let valid_absolute_file_path: &str = "C:/Users/test.txt";

        #[cfg(target_family = "unix")]
        let valid_absolute_file_path: &str = "/home/test.txt";

        assert!(check_if_parent_folder_exists(valid_relative_file_path));
        assert!(!check_if_parent_folder_exists(invalid_file_path));
        assert!(check_if_parent_folder_exists(valid_absolute_file_path));
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_create_folder_if_not_exists() {
        let valid_folder: &str = "./test_folder/";

        assert!(create_folder_if_not_exists(valid_folder).is_ok());
        std::fs::remove_dir(valid_folder).unwrap();
    }

    #[test]
    fn test_get_elapsed_time() {
        let start_time: Instant = Instant::now();
        std::thread::sleep(Duration::from_secs(2));
        let elapsed_time: String = get_elapsed_time(start_time);

        assert_eq!(elapsed_time, "2 seconds");
    }

    #[test]
    fn test_save_passwd_to_file() {
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("./test.txt").unwrap()));
        let passwords: String = "test".to_string();
        assert!(save_passwd_to_file(file, passwords).is_ok());

        let content: String = std::fs::read_to_string("./test.txt").unwrap();
        assert_eq!(content, "test\n");

        std::fs::remove_file("./test.txt").unwrap();
    }

    #[test]
    fn test_get_progress_bar() {
        let pb: ProgressBar = get_progress_bar();
        assert_eq!(pb.length(), Some(100));
    }

    #[test]
    fn test_get_estimated_size() {
        let nb_of_passwords: u64 = 1000;
        let length: u64 = 10;
        assert_eq!(get_estimated_size(nb_of_passwords, length), "10.74 KB");

        let nb_of_passwords: u64 = 1000000;
        let length: u64 = 10;
        assert_eq!(get_estimated_size(nb_of_passwords, length), "10.49 MB");

        let nb_of_passwords: u64 = 0;
        let length: u64 = 10;
        assert_eq!(get_estimated_size(nb_of_passwords, length), "");

        let nb_of_passwords: u64 = 10;
        let length: u64 = 0;
        assert_eq!(get_estimated_size(nb_of_passwords, length), "");
    }
}
