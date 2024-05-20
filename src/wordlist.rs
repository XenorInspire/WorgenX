// Internal crates
use crate::{
    dict,
    error::{SystemError, WorgenXError},
    system,
};

// External crates
use indicatif::ProgressBar;
use std::{
    fs::{File, OpenOptions},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Instant,
};

/// This constant is used to set the size of the buffer used to write the passwords in the file.
/// It specifies the maximum number of passwords that will be written in the file at once per thread.
///
const BUFFER_SIZE: usize = 100000;

/// This static variable is used to keep track of the number of passwords generated.
/// It is used to update the progress bar.
/// It is wrapped in a Mutex to avoid data sharing issues between the threads.
///
static GLOBAL_COUNTER: Mutex<u64> = Mutex::new(0);

/// This struct is built from the user's choices will be used to generate the wordlist.
///
#[derive(Debug)]
pub struct WordlistValues {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub mask: String,
}

/// This struct is built from the WordlistValues struct and will be used to generate the wordlist.
///
#[derive(Debug)]
pub struct WordlistConfig {
    pub dict: Vec<u8>,
    pub mask_indexes: Vec<usize>,
    pub formated_mask: Vec<char>,
}

/// This function is charged to build the final dictionary from the user's choices.
/// It returns a vector of u8 containing the characters that will be used to generate the wordlist.
///
/// # Arguments
///
/// * `wordlist_values` - The struct containing the user's values.
///
/// # Returns
///
/// The vector of u8 containing the characters that will be used to generate the wordlist.
///
fn create_wordlist_content(wordlist_values: &WordlistValues) -> Vec<u8> {
    let mut final_dict: Vec<u8> = Vec::new();

    if wordlist_values.uppercase {
        final_dict.extend(dict::UPPERCASE);
    }

    if wordlist_values.lowercase {
        final_dict.extend(dict::LOWERCASE);
    }

    if wordlist_values.numbers {
        final_dict.extend(dict::NUMBERS);
    }

    if wordlist_values.special_characters {
        final_dict.extend(dict::SPECIAL_CHARACTERS);
    }

    final_dict
}

/// This function is charged to build to format the mask into a vector of char and indexes.
/// This will be used to generate the wordlist.
///
/// # Arguments
///
/// * `mask` - The mask provided by the user.
///
/// # Returns
///
/// A tuple containing the vector of char (formated_mask) and the vector of indexes (mask_indexes).
///
fn format_mask_to_indexes(mask: &str) -> (Vec<char>, Vec<usize>) {
    let mut mask_indexes: Vec<usize> = Vec::new();
    let mut formated_mask: Vec<char> = Vec::new();
    let mut escaped: bool = false;
    let mut idx_formated_mask: usize = 0;
    for c in mask.chars() {
        match c {
            '\\' => {
                if escaped {
                    escaped = false;
                    formated_mask.push(c);
                } else {
                    escaped = true;
                    continue;
                }
            }
            '?' => {
                if escaped {
                    escaped = false;
                    formated_mask.push(c);
                } else {
                    mask_indexes.push(idx_formated_mask);
                    formated_mask.push(0 as char);
                }
            }
            _ => {
                formated_mask.push(c);
            }
        }
        idx_formated_mask += 1;
    }

    (formated_mask, mask_indexes)
}

/// This function is charged to build the WordlistValues struct from the user's values.
///
/// # Arguments
///
/// * `wordlist_values` - The struct containing the user's values.
///
/// # Returns
///
/// The WordlistConfig struct containing the settings of the wordlist.
///
pub fn build_wordlist_config(wordlist_values: &WordlistValues) -> WordlistConfig {
    let dict: Vec<u8> = create_wordlist_content(wordlist_values);
    let (formated_mask, mask_indexes) = format_mask_to_indexes(&wordlist_values.mask);
    WordlistConfig {
        dict,
        mask_indexes,
        formated_mask,
    }
}

/// This function is charged to schedule the wordlist generation.
///
/// # Arguments
///
/// * `wordlist_config` - The WordlistConfig struct containing the settings of the wordlist.
/// * `nb_of_passwords` - The number of passwords to generate.
/// * `nb_of_threads` - The number of threads to use.
/// * `file_path` - The path of the file where the wordlist will be saved.
/// * `no_loading_bar` - A boolean to specify if the loading bar should be displayed or not.
///
/// # Returns
///
/// Ok(()) if the wordlist generation is successful, WorgenXError otherwise.
///
pub fn wordlist_generation_scheduler(
    wordlist_config: &WordlistConfig,
    nb_of_passwords: u64,
    nb_of_threads: u8,
    file_path: &str,
    no_loading_bar: bool,
) -> Result<(), WorgenXError> {
    let pb: Arc<Mutex<indicatif::ProgressBar>> = Arc::new(Mutex::new(system::get_progress_bar()));
    let pb_clone: Arc<Mutex<indicatif::ProgressBar>> = Arc::clone(&pb);
    let start: Instant = Instant::now();
    let main_thread: JoinHandle<Result<(), WorgenXError>> = thread::spawn(move || {
        let mut current_value: u64 = 0;
        while current_value < nb_of_passwords {
            if let Ok(global_counter) = GLOBAL_COUNTER.lock() {
                current_value = *global_counter;
                if !no_loading_bar {
                    build_wordlist_progress_bar(current_value, nb_of_passwords, &pb_clone);
                }
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
    });

    match run_wordlist_generation(wordlist_config, nb_of_passwords, nb_of_threads, file_path) {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    };
    match main_thread.join() {
        Ok(_) => (),
        Err(e) => {
            if let Some(err) = e.downcast_ref::<WorgenXError>() {
                return Err(err.clone());
            } else {
                return Err(WorgenXError::SystemError(SystemError::ThreadError(
                    format!("{:?}", e),
                )));
            }
        }
    }
    println!(
        "\nWordlist generated in {}",
        system::get_elapsed_time(start)
    );
    Ok(())
}

/// This function is charged to start the wordlist generation and dispatch the work between the threads.
///
/// # Arguments
///
/// * `wordlist_config` - The WordlistConfig struct containing the settings of the wordlist.
/// * `nb_of_passwords` - The number of passwords to generate.
/// * `nb_of_threads` - The number of threads to use.
/// * `file_path` - The path of the file where the wordlist will be saved.
///
/// # Returns
///
/// Ok(()) if the wordlist generation is successful, WorgenXError otherwise.
///
fn run_wordlist_generation(
    wordlist_config: &WordlistConfig,
    nb_of_passwords: u64,
    nb_of_threads: u8,
    file_path: &str,
) -> Result<(), WorgenXError> {
    let shared_formated_mask: Arc<Vec<char>> = Arc::new(wordlist_config.formated_mask.clone());
    let shared_mask_indexes: Arc<Vec<usize>> = Arc::new(wordlist_config.mask_indexes.clone());
    let dict_size: usize = wordlist_config.dict.len();
    let shared_dict: Arc<Vec<u8>> = Arc::new(wordlist_config.dict.clone());

    let file: File = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
    {
        Ok(file) => file,
        Err(_) => {
            return Err(WorgenXError::SystemError(SystemError::UnableToCreateFile(
                file_path.to_string(),
                "Please check the path and try again".to_string(),
            )))
        }
    };

    let shared_file: Arc<Mutex<File>> = Arc::new(Mutex::new(file));
    let mut threads: Vec<JoinHandle<Result<(), WorgenXError>>> = Vec::new();
    let dict_indexes: Vec<usize> = vec![0; wordlist_config.mask_indexes.len()];
    let mut nb_of_passwd_per_thread: u64 = nb_of_passwords / nb_of_threads as u64;
    let nb_of_passwd_last_thread: u64 = nb_of_passwd_per_thread + nb_of_passwords % nb_of_threads as u64;
    let mut temp: Vec<usize> = dict_indexes.clone();

    for i in 0..nb_of_threads {
        if i == nb_of_threads - 1 {
            nb_of_passwd_per_thread = nb_of_passwd_last_thread;
        }

        let shared_formated_mask: Arc<Vec<char>> = Arc::clone(&shared_formated_mask);
        let shared_mask_indexes: Arc<Vec<usize>> = Arc::clone(&shared_mask_indexes);
        let shared_dict: Arc<Vec<u8>> = Arc::clone(&shared_dict);
        let file: Arc<Mutex<File>> = Arc::clone(&shared_file);
        let temp_clone: Vec<usize> = temp.clone();
        let thread: JoinHandle<Result<(), WorgenXError>> = thread::spawn(move || {
            match generate_wordlist_part(
                nb_of_passwd_per_thread,
                temp_clone,
                &shared_formated_mask,
                &shared_mask_indexes,
                &shared_dict,
                file,
            ) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });
        threads.push(thread);

        for _ in 0..nb_of_passwd_per_thread {
            for idx in (0..temp.len()).rev() {
                if temp[idx] < dict_size - 1 {
                    temp[idx] += 1;
                    break;
                }
                temp[idx] = 0;
            }
        }
    }

    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            Err(_) => {
                return Err(WorgenXError::SystemError(SystemError::ThreadError(
                    "wordlist generation".to_string(),
                )))
            }
        }
    }

    Ok(())
}

/// This function is charged to generate a part of the wordlist or the whole wordlist if there is only one thread.
/// It returns nothing because it writes the passwords in the file and sends the progress through the channel.
///
/// # Arguments
///
/// * `nb_of_passwd` - The number of passwords to generate.
/// * `dict_indexes` - The indexes of the dictionary.
/// * `formated_mask` - The final mask.
/// * `mask_indexes` - The indexes of the mask.
/// * `dict` - The dictionary.
/// * `file` - The file to write to, wrapped in an `Arc<Mutex<File>>`.
///
/// # Returns
///
/// Ok(()) if the wordlist generation is successful, WorgenXError otherwise.
///
fn generate_wordlist_part(
    nb_of_passwords: u64,
    mut dict_indexes: Vec<usize>,
    formated_mask: &[char],
    mask_indexes: &[usize],
    dict: &[u8],
    file: Arc<Mutex<File>>,
) -> Result<(), WorgenXError> {
    let mut buffer: Vec<String> = Vec::new();
    let mut line: Vec<char> = Vec::with_capacity(formated_mask.len());
    for _ in 0..nb_of_passwords {
        line.clear();

        (0..formated_mask.len()).for_each(|i| {
            let mut found: bool = false;
            for idx in 0..mask_indexes.len() {
                if i == mask_indexes[idx] {
                    found = true;
                    line.push(dict[dict_indexes[idx]] as char);
                    break;
                }
            }

            if !found {
                line.push(formated_mask[i])
            }
        });
        for idx in (0..dict_indexes.len()).rev() {
            if dict_indexes[idx] < dict.len() - 1 {
                dict_indexes[idx] += 1;
                break;
            }
            dict_indexes[idx] = 0;
        }

        buffer.push(line.iter().collect());
        let mut counter = match GLOBAL_COUNTER.lock() {
            Ok(counter) => counter,
            Err(e) => {
                return Err(WorgenXError::SystemError(SystemError::ThreadError(
                    e.to_string(),
                )))
            }
        };
        *counter += 1;

        if buffer.len() == BUFFER_SIZE {
            match system::save_passwd_to_file(Arc::clone(&file), buffer.join("\n")) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        match system::save_passwd_to_file(Arc::clone(&file), buffer.join("\n")) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(())
}

/// This function is charged to build the progress bar during the wordlist generation.
/// It returns nothing because it juste updates the progress bar.
///
/// # Arguments
///
/// * `nb_of_passwd_generated` - The number of passwords generated.
/// * `total_nb_of_passwd` - The total number of passwords to generate.
/// * `pb` - The progress bar instance (from the indicatif crate).
///
pub fn build_wordlist_progress_bar(
    nb_of_passwd_generated: u64,
    total_nb_of_passwd: u64,
    pb: &Arc<Mutex<ProgressBar>>,
) {
    let mut pourcentage: u64 = (nb_of_passwd_generated * 100) / total_nb_of_passwd;
    if pourcentage == 0 {
        pourcentage += 1;
    }
    if let Ok(pb) = pb.try_lock() {
        if nb_of_passwd_generated < total_nb_of_passwd {
            pb.set_position(pourcentage);
            pb.set_message("Loading...");
        } else {
            pb.set_position(100);
            pb.finish_with_message(String::from("Wordlist generated"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wordlist_content() {
        let wordlist_values: WordlistValues = WordlistValues {
            numbers: true,
            special_characters: true,
            uppercase: true,
            lowercase: true,
            mask: String::from("????"),
            hash: String::from(""),
        };
        let result: Vec<u8> = create_wordlist_content(&wordlist_values);
        assert_eq!(result.len(), 91);
    }

    #[test]
    fn test_format_mask_to_indexes() {
        let mask: String = String::from("????");
        let (formated_mask, mask_indexes) = format_mask_to_indexes(&mask);
        assert_eq!(formated_mask, vec!['\0', '\0', '\0', '\0']);
        assert_eq!(mask_indexes, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_build_wordlist_config() {
        let wordlist_values: WordlistValues = WordlistValues {
            numbers: true,
            special_characters: true,
            uppercase: true,
            lowercase: true,
            mask: String::from("????"),
            hash: String::from(""),
        };
        let wordlist_config: WordlistConfig = build_wordlist_config(&wordlist_values);
        assert_eq!(wordlist_config.dict.len(), 91);
        assert_eq!(wordlist_config.mask_indexes, vec![0, 1, 2, 3]);
        assert_eq!(wordlist_config.formated_mask, vec!['\0', '\0', '\0', '\0']);
    }

    #[test]
    fn test_generate_wordlist_part() {
        let nb_of_passwords: u64 = 10;
        let dict_indexes: Vec<usize> = vec![0, 0, 0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0', '\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1, 2, 3];
        let dict: Vec<u8> = vec![b'a', b'b', b'c', b'd'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            Arc::clone(&file),
            "",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test.txt").unwrap();
        let expected_content: String = String::from("aaaa\naaab\naaac\naaad\naaba\naabb\naabc\naabd\naaca\naacb\n");
        assert_eq!(content.lines().count(), 10);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test.txt").unwrap();
    }
}
