// Internal crates
use crate::{
    dict,
    error::{SystemError, WorgenXError},
    system,
};

// External crates
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::{File, OpenOptions},
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
};

/// This constant is used to set the size of the buffer used to write the passwords in the file
/// It specifies the maximum number of passwords that will be written in the file at once per thread
///
const BUFFER_SIZE: usize = 100000;

/// This struct is built from the user's choices will be used to generate the wordlist
///
pub struct WordlistValues {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub mask: String,
}

/// This struct is built from the WordlistValues struct and will be used to generate the wordlist
///
pub struct WordlistConfig {
    pub dict: Vec<u8>,
    pub mask_indexes: Vec<usize>,
    pub formated_mask: Vec<char>,
}

/// This function is charged to build the final dictionary from the user's choices
/// It returns a vector of u8 containing the characters that will be used to generate the wordlist
///
/// # Arguments
///
/// * `wordlist_values` - The struct containing the user's values
///
/// # Returns
///
/// The vector of u8 containing the characters that will be used to generate the wordlist
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

/// This function is charged to build to format the mask into a vector of char and indexes
/// This will be used to generate the wordlist
///
/// # Arguments
///
/// * `mask` - The mask provided by the user
///
/// # Returns
///
/// A tuple containing the vector of char (formated_mask) and the vector of indexes (mask_indexes)
///
fn format_mask_to_indexes(mask: &str) -> (Vec<char>, Vec<usize>) {
    let mut mask_indexes: Vec<usize> = Vec::new();
    let mut formated_mask: Vec<char> = Vec::new();
    let mut escaped = false;
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

/// This function is charged to build the WordlistValues struct from the user's values
///
/// # Arguments
///
/// * `wordlist_values` - The struct containing the user's values
///
/// # Returns
///
/// The WordlistConfig struct containing the settings of the wordlist
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

/// This function is charged to schedule the wordlist generation (WIP)
///
/// # Arguments
///
/// * `wordlist_config` - The WordlistConfig struct containing the settings of the wordlist
/// * `wordlist_values` - The WordlistValues struct containing the user's values
/// * `nb_of_passwords` - The number of passwords to generate
/// * `nb_of_threads` - The number of threads to use
/// * `tx` - The channel sender for the progress bar and/or errors
///
/// # Returns
///
/// On success, it returns the time elapsed to generate the wordlist
/// On error, it returns a WordlistError
///
pub fn wordlist_generation_scheduler(
    wordlist_config: &WordlistConfig,
    nb_of_passwords: u64,
    nb_of_threads: u64,
    file_path: &str,
    tx: &Sender<Result<u64, WorgenXError>>,
) -> Result<(), WorgenXError> {
    let shared_formated_mask = Arc::new(wordlist_config.formated_mask.clone());
    let shared_mask_indexes = Arc::new(wordlist_config.mask_indexes.clone());
    let dict_size = wordlist_config.dict.len();
    let shared_dict = Arc::new(wordlist_config.dict.clone());

    let file = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
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
    let shared_file = Arc::new(Mutex::new(file));

    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let dict_indexes: Vec<usize> = vec![0; wordlist_config.mask_indexes.len()];
    let mut nb_of_passwd_per_thread = nb_of_passwords / nb_of_threads;
    let nb_of_passwd_last_thread = nb_of_passwd_per_thread + nb_of_passwords % nb_of_threads;
    let mut temp = dict_indexes.clone();

    for i in 0..nb_of_threads {
        if i == nb_of_threads - 1 {
            nb_of_passwd_per_thread = nb_of_passwd_last_thread;
        }

        let shared_formated_mask = Arc::clone(&shared_formated_mask);
        let shared_mask_indexes = Arc::clone(&shared_mask_indexes);
        let shared_dict = Arc::clone(&shared_dict);
        let file = Arc::clone(&shared_file);
        let temp_clone = temp.clone();
        let tx_clone = tx.clone();
        let thread = thread::spawn(move || {
            generate_wordlist_part(
                nb_of_passwd_per_thread,
                temp_clone,
                shared_formated_mask,
                shared_mask_indexes,
                shared_dict,
                file,
                &tx_clone,
            );
        });
        threads.push(thread);

        for _ in 0..nb_of_passwd_per_thread {
            for idx in (0..temp.len()).rev() {
                if temp[idx] < dict_size - 1 {
                    temp[idx] += 1;
                    break;
                } else {
                    temp[idx] = 0;
                }
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

/// This function is charged to generate a part of the wordlist or the whole wordlist if there is only one thread
/// It can send possible errors through the channel
///
/// # Arguments
///
/// * `nb_of_passwd` - The number of passwords to generate
/// * `dict_indexes` - The indexes of the dictionary
/// * `formated_mask` - The final mask
/// * `mask_indexes` - The indexes of the mask
/// * `dict` - The dictionary
/// * `file` - The file to write to, wrapped in an Arc<Mutex<File>>
/// * `tx` - The channel sender for the progress bar and/or errors
///
fn generate_wordlist_part(
    nb_of_passwords: u64,
    mut dict_indexes: Vec<usize>,
    formated_mask: Arc<Vec<char>>,
    mask_indexes: Arc<Vec<usize>>,
    dict: Arc<Vec<u8>>,
    file: Arc<Mutex<File>>,
    tx: &Sender<Result<u64, WorgenXError>>,
) {
    let mut buffer: Vec<String> = Vec::new();
    for _ in 0..nb_of_passwords {
        let mut line = String::new();
        (0..formated_mask.len()).for_each(|i| {
            let mut found = false;
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
            } else {
                dict_indexes[idx] = 0;
            }
        }

        buffer.push(line);
        tx.send(Ok(1)).unwrap_or(());
        if buffer.len() == BUFFER_SIZE {
            match system::save_passwd_to_file(Arc::clone(&file), buffer.join("\n")) {
                Ok(_) => {}
                Err(e) => {
                    tx.send(Err(e)).unwrap_or(());
                }
            }
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        match system::save_passwd_to_file(Arc::clone(&file), buffer.join("\n")) {
            Ok(_) => {}
            Err(e) => {
                tx.send(Err(e)).unwrap_or(());
            }
        }
    }
}

/// This function is charged to build the progress bar during the wordlist generation
///
/// # Arguments
///
/// * `nb_of_passwd_generated` - The number of passwords generated
/// * `total_nb_of_passwd` - The total number of passwords to generate
/// * `pb` - The progress bar instance (from the indicatif crate)
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
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.green}] {pos:>7}% | {msg}")
                    .unwrap_or(ProgressStyle::default_bar()) // Provide the default argument
                    .progress_chars("##-"),
            );

            pb.set_position(pourcentage);
            pb.set_message("Loading...");
        } else {
            pb.set_position(100);

            pb.finish_with_message(String::from("Wordlist generated"));
        }
    }
}
