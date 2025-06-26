// Internal crates.
use crate::error::{SystemError, WorgenXError};

// External crates.
use blake2::{Blake2b512, Blake2s256};
use digest::Digest;
use indicatif::{ProgressBar, ProgressStyle};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use whirlpool::Whirlpool;

#[cfg(feature = "gui")]
use std::{cmp::PartialOrd, default::Default, fmt::Display, io::stdin, marker::Copy, str::FromStr};

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

/// This function is responsible for getting user String input y/n.
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

/// This function is responsible for geting user String input.
///
/// # Returns
///
/// The value entered by the user. If an error occurs, the function returns an empty String.
///
#[cfg(feature = "gui")]
pub fn get_user_choice() -> String {
    let mut buffer: String = String::new();
    if stdin().read_line(&mut buffer).is_ok() {
        buffer.trim().to_string()
    } else {
        String::new()
    }
}

/// This function is responsible for getting user int input.
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
            Err(e) => println!("Please enter a valid number greater than 0, {e}"),
        }
    }

    number
}

/// This function is responsible for checking a path/filename.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path/filename to check.
///
/// # Returns
///
/// Ok(String) if the path/filename is valid, containing the full path, SystemError otherwise.
///
pub fn is_valid_path(path: &str) -> Result<String, SystemError> {
    let filename: String = match Path::new(path).file_name() {
        Some(f) => match f.to_str() {
            Some(f) => f.to_string(),
            None => return Err(SystemError::InvalidPath(path.to_string())),
        },
        None => return Err(SystemError::InvalidPath(path.to_string())),
    };

    let invalid_chars: &[char] = get_invalid_chars();
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(SystemError::InvalidFilename(filename));
    }

    let full_path: String = if Path::new(path).is_absolute() {
        path.to_string()
    } else {
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
        current_dir + "/" + path.trim_start_matches("./")
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

/// This function is responsible for checking if the parent folder exists from a given file path.
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
    Path::new(file_path).parent().is_some_and(Path::exists)
}

/// This function is responsible for creating the passwords or wordlists folder if it doesn't exist.
///
/// # Arguments
///
/// * `folder` - A string slice that holds the folder to create.
///
/// # Returns
///
/// Ok(()) if the folder has been created, SystemError otherwise.
///
#[cfg(feature = "gui")]
pub fn create_folder_if_not_exists(folder: &str) -> Result<(), SystemError> {
    let mut folder: String = String::from(folder);
    if folder.pop().is_none() {
        return Err(SystemError::InvalidPath(folder));
    }
    if !Path::new(&folder).exists() {
        std::fs::create_dir_all(&folder)
            .map_err(|e| SystemError::UnableToCreateFolder(folder, e.to_string()))?;
    }

    Ok(())
}

/// This function sends the invalid chars for windows platforms.
///
/// # Returns
///
/// '<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n' chars.
///
#[cfg(target_family = "windows")]
const fn get_invalid_chars() -> &'static [char] {
    &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '+', ',', ';', '=', '@', '\0', '\r', '\n',]
}

/// This function sends the invalid chars for unix platforms.
///
/// # Returns
///
/// '/', '\0', '\r', '\n' chars.
///
#[cfg(target_family = "unix")]
const fn get_invalid_chars() -> &'static [char] {
    &['/', '\0', '\r', '\n']
}

/// This function is responsible for calculating the elapsed time between two timestamps.
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
        elapsed_time_str.push_str(" second(s)");
    }

    elapsed_time_str
}

/// This function is responsible for saving the generated passwords in a file and sends the progress/possible errors to the channel.
///
/// # Arguments
///
/// * `file` - The file to write to, wrapped in an `Arc<Mutex<File>>`.
/// * `passwords` - The passwords to write.
///
/// # Returns
///
/// Ok(()) if the passwords have been written to the file, WorgenXError otherwise.
///
pub fn save_passwd_to_file(file: &Arc<Mutex<File>>, passwords: &str) -> Result<(), WorgenXError> {
    let mut file = file.lock().map_err(|_| {
        WorgenXError::SystemError(SystemError::UnableToWriteToFile(
            "output file".to_string(),
            "Please check the path, the permissions and try again".to_string(),
        ))
    })?;

    file.write_all(format!("{passwords}\n").as_bytes())
        .map_err(|_| {
            WorgenXError::SystemError(SystemError::UnableToWriteToFile(
                "output file".to_string(),
                "Please check the path, the permissions and try again".to_string(),
            ))
        })
}

/// This function is responsible for returniong the progress used by the program.
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
            .unwrap_or_else(|_| ProgressStyle::default_bar()) // Provide the default argument
            .progress_chars("##-"),
    );
    pb
}

/// This function is responsible for returning the estimated size of the wordlist.
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

/// This function is responsible for managing password hashing.
/// It returns the hashed password from the hash algorithm specified by the user.
/// If the hash algorithm is not supported, it returns an error.
///
/// # Arguments
///
/// * `password` - The password to hash.
/// * `hash` - The hash algorithm to use.
///
/// # Returns
///
/// The hashed password, SystemError otherwise.
///
pub fn manage_hash(password: &str, hash: &str) -> Result<String, SystemError> {
    match hash {
        "md5" => Ok(hash_with_digest(Md5::new(), password)),
        "sha1" => Ok(hash_with_digest(Sha1::new(), password)),
        "sha224" => Ok(hash_with_digest(Sha224::new(), password)),
        "sha256" => Ok(hash_with_digest(Sha256::new(), password)),
        "sha384" => Ok(hash_with_digest(Sha384::new(), password)),
        "sha512" => Ok(hash_with_digest(Sha512::new(), password)),
        "sha3-224" => Ok(hash_with_digest(Sha3_224::new(), password)),
        "sha3-256" => Ok(hash_with_digest(Sha3_256::new(), password)),
        "sha3-384" => Ok(hash_with_digest(Sha3_384::new(), password)),
        "sha3-512" => Ok(hash_with_digest(Sha3_512::new(), password)),
        "blake2b-512" => Ok(hash_with_digest(Blake2b512::new(), password)),
        "blake2s-256" => Ok(hash_with_digest(Blake2s256::new(), password)),
        "whirlpool" => Ok(hash_with_digest(Whirlpool::new(), password)),
        _ => Err(SystemError::UnsupportedHashAlgorithm(hash.to_string())),
    }
}

/// This function is responsible for hashing a password with a specific hash algorithm.
/// It returns the hashed password.
///
/// # Arguments
///
/// * `hasher` - The hasher to use, it must implement the Digest trait.
/// * `password` - The password to hash.
///
/// # Returns
///
/// The hashed password.
///
fn hash_with_digest<D: Digest>(mut hasher: D, password: &str) -> String {
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_path() {
        let relative_path: &str = "./test.txt";
        let invalid_path: &str = "test.txt\0";

        #[cfg(target_family = "windows")]
        let absolute_path: &str = "C:/Users/test.txt";

        #[cfg(target_family = "unix")]
        let absolute_path: &str = "/home/test.txt";

        assert!(is_valid_path(relative_path).is_ok());
        assert!(is_valid_path(absolute_path).is_ok());
        assert!(is_valid_path(invalid_path).is_err());
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

        assert_eq!(elapsed_time, "2 second(s)");
    }

    #[test]
    fn test_save_passwd_to_file() {
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("./test.txt").unwrap()));
        let passwords: String = "test".to_string();
        assert!(save_passwd_to_file(&file, &passwords).is_ok());

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

    #[test]
    fn test_hash_password() {
        let password: &str = "password";

        assert_eq!(manage_hash(password, "md5").unwrap(), "5f4dcc3b5aa765d61d8327deb882cf99");
        assert_eq!(manage_hash(password, "sha1").unwrap(), "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8" );
        assert_eq!(manage_hash(password, "sha224").unwrap(), "d63dc919e201d7bc4c825630d2cf25fdc93d4b2f0d46706d29038d01");
        assert_eq!(manage_hash(password, "sha256").unwrap(), "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8");
        assert_eq!(manage_hash(password, "sha384").unwrap(), "a8b64babd0aca91a59bdbb7761b421d4f2bb38280d3a75ba0f21f2bebc45583d446c598660c94ce680c47d19c30783a7");
        assert_eq!(manage_hash(password, "sha512").unwrap(), "b109f3bbbc244eb82441917ed06d618b9008dd09b3befd1b5e07394c706a8bb980b1d7785e5976ec049b46df5f1326af5a2ea6d103fd07c95385ffab0cacbc86");
        assert_eq!(manage_hash(password, "sha3-224").unwrap(), "c3f847612c3780385a859a1993dfd9fe7c4e6d7f477148e527e9374c");
        assert_eq!(manage_hash(password, "sha3-256").unwrap(), "c0067d4af4e87f00dbac63b6156828237059172d1bbeac67427345d6a9fda484");
        assert_eq!(manage_hash(password, "sha3-384").unwrap(), "9c1565e99afa2ce7800e96a73c125363c06697c5674d59f227b3368fd00b85ead506eefa90702673d873cb2c9357eafc");
        assert_eq!(manage_hash(password, "sha3-512").unwrap(), "e9a75486736a550af4fea861e2378305c4a555a05094dee1dca2f68afea49cc3a50e8de6ea131ea521311f4d6fb054a146e8282f8e35ff2e6368c1a62e909716");
        assert_eq!(manage_hash(password, "blake2s-256").unwrap(), "4c81099df884bd6e14a639d648bccd808512e48af211ae4f44d545ea6d5e5f2b");
        assert_eq!(manage_hash(password, "blake2b-512").unwrap(), "7c863950ac93c93692995e4732ce1e1466ad74a775352ffbaaf2a4a4ce9b549d0b414a1f3150452be6c7c72c694a7cb46f76452917298d33e67611f0a42addb8");
        assert_eq!(manage_hash(password, "whirlpool").unwrap(), "74dfc2b27acfa364da55f93a5caee29ccad3557247eda238831b3e9bd931b01d77fe994e4f12b9d4cfa92a124461d2065197d8cf7f33fc88566da2db2a4d6eae");
        assert!(manage_hash(password, "sha999").is_err());
    }
}
