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
    sync::{Arc, Mutex, MutexGuard},
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
    pub hash: String,
}

/// This struct is built from the WordlistValues struct and will be used to generate the wordlist.
///
#[derive(Debug)]
pub struct WordlistConfig {
    pub dict: Vec<u8>,
    pub mask_indexes: Vec<usize>,
    pub formated_mask: Vec<char>,
    pub hash: String,
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
        hash: wordlist_values.hash.clone(),
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

    let file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .map_err(|_| WorgenXError::SystemError(SystemError::UnableToCreateFile(
            file_path.to_string(),
            "Please check the path and try again".to_string(),
        )))?;

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
        let shared_hash: String = wordlist_config.hash.clone();
        let file: Arc<Mutex<File>> = Arc::clone(&shared_file);
        let temp_clone: Vec<usize> = temp.clone();
        let thread: JoinHandle<Result<(), WorgenXError>> = thread::spawn(move || {
            generate_wordlist_part(
                nb_of_passwd_per_thread,
                temp_clone,
                &shared_formated_mask,
                &shared_mask_indexes,
                &shared_dict,
                file,
                &shared_hash,
            )
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
/// * `hash` - The hash algorithm to use, if any.
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
    hash: &str,
) -> Result<(), WorgenXError> {
    let mut buffer: Vec<String> = Vec::new();
    let mut line: Vec<char> = Vec::with_capacity(formated_mask.len());

    // This closure is used to hash the password if the user has specified a hash algorithm.
    let process_line: Box<dyn Fn(String) -> Result<String, WorgenXError>> = if !hash.is_empty() {
        Box::new(|line_str: String| -> Result<String, WorgenXError> {
            match system::manage_hash(line_str, hash) {
                Ok(hashed_passwd) => Ok(hashed_passwd),
                Err(e) => Err(WorgenXError::SystemError(e)),
            }
        })
    } else {
        Box::new(|line_str: String| -> Result<String, WorgenXError> { Ok(line_str) })
    };

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

        buffer.push(process_line(line.iter().collect::<String>())?);

        let mut counter: MutexGuard<u64> = match GLOBAL_COUNTER.lock() {
            Ok(counter) => counter,
            Err(e) => {
                return Err(WorgenXError::SystemError(SystemError::ThreadError(
                    e.to_string(),
                )))
            }
        };
        *counter += 1;

        if buffer.len() == BUFFER_SIZE {
            system::save_passwd_to_file(Arc::clone(&file), buffer.join("\n"))?;
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        system::save_passwd_to_file(Arc::clone(&file), buffer.join("\n"))?;
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
    fn test_generate_wordlist_part_without_hash() {
        let nb_of_passwords: u64 = 10;
        let dict_indexes: Vec<usize> = vec![0, 0, 0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0', '\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1, 2, 3];
        let dict: Vec<u8> = vec![b'a', b'b', b'c', b'd'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test1.txt").unwrap()));
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

        let content: String = std::fs::read_to_string("test1.txt").unwrap();
        let expected_content: String = String::from("aaaa\naaab\naaac\naaad\naaba\naabb\naabc\naabd\naaca\naacb\n");
        assert_eq!(content.lines().count(), 10);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test1.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha1_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test2.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            Arc::clone(&file),
            "sha1",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test2.txt").unwrap();
        let expected_content: String = String::from(
            "fb96549631c835eb239cd614cc6b5cb7d295121a
ddfe163345d338193ac2bdc183f8e9dcff904b43
bcac9d1d8eab3713ae489224d0130c9468e7a0e3
3ea6c91e241f256e5e3a88ebd647372022323a53
b1d5781111d84f7b3fe45a0852e59758cd7a87e5
17ba0791499db908433b80f37c5fbc89b870084b
7b52009b64fd0a2a49e6d8a939753077792b0554
bd307a3ec329e10a2cff8fb87480823da114f8f4
91032ad7bbcb6cf72875e8e8207dcfba80173f7c
472b07b9fcf2c2451e8781e944bf5f77cd8457c8
12c6fc06c99a462375eeb3f43dfd832b08ca9e17
d435a6cdd786300dff204ee7c2ef942d3e9034e2
22d200f8670dbdb3e253a90eee5098477c95c23d
632667547e7cd3e0466547863e1207a8c0c0c549
cb4e5208b4cd87268b208e49452ed6e89a68e0b8
b6692ea5df920cad691c20319a6fffd7a4a766b8
",
        );
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test2.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha224_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test3.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            Arc::clone(&file),
            "sha224",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test3.txt").unwrap();
        let expected_content: String = String::from(
            "5538ae2b02d4ae0b7090dc908ca69cd11a2ffad43c7435f1dbad5e6a
67e7c42dcfc51549c366d8f81b52e62e30778aaa99498f44b30d954f
86fd614f8fe38fc1e1c68733b6cd9acba79623f50b6ee81804054a03
1d2bba65c0056963590648b35fba49a8cea4b2597e8953baf66082ad
3aac67cd73162d439f9947d61357a1b62432f0ca84b7f435f4177a8c
161a68601ec1d8ca45250557cff3ffb98eca53fbcf86bbdb8e8bb6e7
3c794f0c67bd561ce841fc6a5999bf0df298a0f0ae3487efda9d0ef4
86730f0dd6381286d3b5f0dfb897ce4895480ce97564c6be4f1543b8
751267062c92e398c3942214b58136f73a4b9e1ca9a214d72d6d5805
112a6cfa3ed398a5248c479ed6a061d397633a3fb4d5b9ecca274695
a29662a4f922a60411ca84c145bf83b76ec59210be23d995e30934d2
bd1a1bdf6eae5ee14c3fee371cca975a5e052009bc67ce8f11cb7271
6332531eeafc6e0ede272192be898f549950fb32b209d04f0a98306a
c05730c3a7a2dbef1693c0299929166dd379d9b57a1ddc5024aa2b48
83dd3a4315c03a802c7ab4bf6b61b2bd847976c1295e8e7284ec59b7
f193e8b0c6e8e436ed6fbff804917367733cddc04514d1865452f399
",
        );
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test3.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha256_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test4.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            Arc::clone(&file),
            "sha256",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test4.txt").unwrap();
        let expected_content: String = String::from(
            "f1534392279bddbf9d43dde8701cb5be14b82f76ec6607bf8d6ad557f60f304e
938db8c9f82c8cb58d3f3ef4fd250036a48d26a712753d2fde5abd03a85cabf4
a953f09a1b6b6725b81956e9ad0b1eb49e3ad40004c04307ef8af6246a054116
0b8efa5a3bf104413a725c6ff0459a6be12b1fd33314cbb138745baf39504ae5
4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5
4fc82b26aecb47d2868c4efbe3581732a3e7cbcc6c2efb32062c08170a05eeb8
6b51d431df5d7f141cbececcf79edf3dd861c3b4069f0b11661a3eefacbba918
3fdba35f04dc8c462986c992bcf875546257113072a909c162f7e470e581e278
f5ca38f748a1d6eaf726b8a42fb575c3c71f1864a8143301782de13da2d9202b
6f4b6612125fb3a0daecd2799dfd6c9c299424fd920f9b308110a2c1fbd8f443
785f3ec7eb32f30b90cd0fcf3657d388b5ff4297f2f9716ff66e9b69c05ddd09
535fa30d7e25dd8a49f1536779734ec8286108d115da5045d77f3b4185d8f790
624b60c58c9d8bfb6ff1886c2fd605d2adeb6ea4da576068201b6c6958ce93f4
eb1e33e8a81b697b75855af6bfcdbcbf7cbbde9f94962ceaec1ed8af21f5a50f
e29c9c180c6279b0b02abd6a1801c7c04082cf486ec027aa13515e4f3884bb6b
c6f3ac57944a531490cd39902d0f777715fd005efac9a30622d5f5205e7f6894
",
        );
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test4.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha384_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test5.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            Arc::clone(&file),
            "sha384",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test5.txt").unwrap();
        let expected_content: String = String::from("34ae2cd40efabf896d8d4173e500278d10671b2d914efb5480e8349190bc7e8e1d532ad568d00a8295ea536a9b42bbc6
0096bbced466ec5d14b1f2fdba6224279ea34a68623076c58d8a58de943d8a1b8d8756ff30424b9e4a21599db0808f57
54e92b186c6c39eba781a4d082fbdf22fe7bd41197a3968d428835ef3826c4f087f5a32af4431ab4398ab4afa583b638
63d3959fe2fa54b770021fbeec692e984c3691269dc0b72e44bc5c955b3a2f1d322980cf84f506be055eb3eea510511a
b1769933399d67ba4d8128a5769841233712ff8b69ac6828285f6c085eb888d052bff30e94f9b280e9808235b0ced7f4
9b20aa6472eef4fd186d231637b1c1d55a5a434cc9130d6afcaaf486253a20c23a4eaeea419594c17f46bc53c7cee12e
1e237288d39d815abc653befcab0eb70966558a5bbc10a24739c116ed2f615be31e81670f02af48fe3cf5112f0fa03e8
6356fbb43627033e886785c2a9c16980336df008b720d23f98b35e06ee69246287739c9d7458b39356c3bdb1e4e2c7fc
fd21efb0c2863b1d2460f7e6048d757beb2326c6e1bbee5194826be2c626a9de3bc8d6f2488617e505c960478d3855d0
395809b7b4da42f70785d72cc237e64c7916fa6b2167c15202c09915071639718369426689502bd6a7b60e24cd90d8e8
1ba40d8a5dcd0f2f0071687f3253f59780a582305a0cee1a49a56a4736dce4fc8af88372c79393a3a569aeda0c15959d
6fda40fc935c39c3894ca91b3faf4acb16fe34d1fc2992c7019f2e35f98fda0aa18b39727f9f0759e6f1cd737ca5c948
32f5039553078543bf8748756a64c8b02338afbc1ee3c70dde5988760c3b8833e0e3c830fea5b65f08cb803842eb6ed6
63eba807c25331f128c471c835447eed6b450dd0b3887234e6810a2ffdbfa30b71c7c286ea35e5e6642827dcf65a7bc8
7ed19d3dedd3e815719b99836042944e36bb281495983599a271ead582f300e304617f756deedc2148766989a4de1c0d
53caf7beda0d6881f85a17efd3e9758f464bab6618478c987b8edabec62c888d59dc9ac1c1111bc9acd7b9d2b9c1c55c
");
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test5.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha512_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test6.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            Arc::clone(&file),
            "sha512",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test6.txt").unwrap();
        let expected_content: String = String::from("8ab3361c051a97ddc3c665d29f2762f8ac4240d08995f8724b6d07d8cbedd32c28f589ccdae514f20a6c8eea6f755408dd3dd6837d66932ca2352eaeab594427
7b3e2f9860391685c2ff6785ab60541bb0db11a20b7e511bf020f4b3073053cc36b647d82f520c5a1c323e853f4bb110fce8210ba409fa42069ab4bf0b0d39e1
c9fb10083ed68061842ecdbd848143964f8c492586e3528aede8ae419450e300136e499b80e46092b5eb5b26db9d8006750ac8c1c3e85b7f1e05319cf6b315e5
081353b00607b812ca63641b911da8f4a5debd68c1d0c6aac95e698d4c58eff8259bdfc6e5f23e4af7924a16dfbbd8292e66d6460641b62c1301d4e80f166f82
3c11e4f316c956a27655902dc1a19b925b8887d59eff791eea63edc8a05454ec594d5eb0f40ae151df87acd6e101761ecc5bb0d3b829bf3a85f5432493b22f37
74a49c698dbd3c12e36b0b287447d833f74f3937ff132ebff7054baa18623c35a705bb18b82e2ac0384b5127db97016e63609f712bc90e3506cfbea97599f46f
5aadb45520dcd8726b2822a7a78bb53d794f557199d5d4abdedd2c55a4bd6ca73607605c558de3db80c8e86c3196484566163ed1327e82e8b6757d1932113cb8
413f2ba78c7ed4ccefbe0cc4f51d3eb5cb15f13fec999de4884be925076746663aa5d34476a3df4a8729fd8eea01defa4f3f66e99bf943f4d84382d64bbbfa9e
dfa5d1cefd0efdf5f52b765120da72c5706eb1dd113234cfdf31e31f9cd0283366f6a8f7230f29ea42d83acfe02743dc2504cda07c30f6e84bf9b1ca35966266
198dabf4bac21cf35cddb48db0f8b67c56b2bdf63767242aea7342fe68c0b9df8d37f3e47a134648e19f1640e158f2e527e636db122a9143307cf309efcb85d9
6ad275d26c200e81534d9996183c8748ddfabc7b0a011a90f46301626d709923474703cacab0ff8b67cd846b6cb55b23a39b03fbdfb5218eec3373cf7010a166
6ff334e1051a09e90127ba4e309e026bb830163a2ce3a355af2ce2310ff6e7e9830d20196a3472bfc8632fd3b60cb56102a84fae70ab1a32942055eb40022225
1ccbff33e55627a50beca8cf5c89f77c3165dcb3218171308423f250f0bb0be9700bbfdd92d35dfa2e579110266a40194d707b50e7d27b6f09b81fbbf80231a3
5305f867c631e8335813a103a4942a93037c3d3b1982eab342fb495047dcc79e13299ab65b5f4a34400f15af384eda2ed7144671e83996334c0669fc8377a130
e63006bd9f35f06cd20582fc8b34ae76a15080297be886decd6dfd42f59e5174a537e8cd92ef577297f967beb6b758c1835f4c270c251e10c12331fcd8635c53
3163a8d6a4540ecf1794ece0245f291154d30e1080359d2e994ef79c1a469aa0cd808769d9c7ee30ca342c6803d2ebcec3eb71a928d6db187dfb1fc2cf640395
");
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test6.txt").unwrap();
    }
}
