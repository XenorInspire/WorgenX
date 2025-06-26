// Internal crates.
use crate::{
    dict,
    error::{SystemError, WorgenXError},
    system,
};

// External crates.
use indicatif::ProgressBar;
use std::{
    fs::{File, OpenOptions},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

/// This constant is used to set the size of the buffer used to write the passwords in the file.
/// It specifies the maximum number of passwords that will be written in the file at once per thread.
///
const BUFFER_SIZE: usize = 100000;

/// This static variable is used to track the number of passwords generated.
/// It is used to update the progress bar.
/// It is wrapped in a AtomicU64 to avoid data sharing issues between the threads.
///
static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// This struct is built from the user's choices and will be used to generate the wordlist.
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

/// This function is responsible for building the final dictionary from the user's choices.
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

/// This function is responsible for converting the mask into a vector of char and indexes.
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
                    formated_mask.push(0u8 as char);
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

/// This function is responsible for building the WordlistValues struct from the user's values.
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

/// This function is responsible for scheduling the wordlist generation.
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
    nb_of_threads: usize,
    file_path: &str,
    no_loading_bar: bool,
) -> Result<(), WorgenXError> {
    let pb: Arc<Mutex<indicatif::ProgressBar>> = Arc::new(Mutex::new(system::get_progress_bar()));
    let pb_clone: Arc<Mutex<indicatif::ProgressBar>> = Arc::clone(&pb);
    let start: Instant = Instant::now();
    
    let main_thread: JoinHandle<Result<(), WorgenXError>> = thread::spawn(move || {
        while GLOBAL_COUNTER.load(Ordering::SeqCst) < nb_of_passwords {
            if !no_loading_bar {
                build_wordlist_progress_bar(GLOBAL_COUNTER.load(Ordering::SeqCst), nb_of_passwords, &pb_clone);
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
    });

    run_wordlist_generation(wordlist_config, nb_of_passwords, nb_of_threads, file_path)?;
    if let Err(e) = main_thread.join() {
        if let Some(err) = e.downcast_ref::<WorgenXError>() {
            return Err(err.clone());
        }

        return Err(WorgenXError::SystemError(SystemError::ThreadError(format!("{e:?}"))));
    }

    println!("\nWordlist generated in {}", system::get_elapsed_time(start));
    Ok(())
}

/// This function is responsible for starting the wordlist generation and dispatches the work between the threads.
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
    nb_of_threads: usize,
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
    let mut temp: Vec<usize> = dict_indexes;

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
                &file,
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
        if thread.join().is_err() {
            return Err(WorgenXError::SystemError(SystemError::ThreadError(
                "wordlist generation".to_string(),
            )))
        }
    }

    Ok(())
}

/// This function is responsible for generating a part of the wordlist or the whole wordlist if there is only one thread.
/// It sends the progress value through the channel (to update the progess bar).
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
    file: &Arc<Mutex<File>>,
    hash: &str,
) -> Result<(), WorgenXError> {
    let mut buffer: Vec<String> = Vec::new();
    let mut line: Vec<char> = Vec::with_capacity(formated_mask.len());

    // This closure is used to hash the password if the user has specified a hash algorithm.
    let process_line: Box<dyn Fn(String) -> Result<String, WorgenXError>> = if hash.is_empty() {
        Box::new(|line_str: String| -> Result<String, WorgenXError> { Ok(line_str) })
    } else {
        Box::new(|line_str: String| -> Result<String, WorgenXError> {
            match system::manage_hash(&line_str, hash) {
                Ok(hashed_passwd) => Ok(hashed_passwd),
                Err(e) => Err(WorgenXError::SystemError(e)),
            }
        })
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
        GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);

        if buffer.len() == BUFFER_SIZE {
            system::save_passwd_to_file(&Arc::clone(file), &buffer.join("\n"))?;
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        system::save_passwd_to_file(&Arc::clone(file), &buffer.join("\n"))?;
    }
    Ok(())
}

/// This function is responsible for building the progress bar during the wordlist generation.
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
            &Arc::clone(&file),
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
    fn test_generate_wordlist_part_with_md5_hash() {
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
            &Arc::clone(&file),
            "md5",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test2.txt").unwrap();
        let expected_content: String = String::from(
            "b4b147bc522828731f1a016bfa72c073
96a3be3cf272e017046d1b2674a52bd3
a2ef406e2c2351e0b9e80029c909242d
e45ee7ce7e88149af8dd32b27f9512ce
d3d9446802a44259755d38e6d163e820
6512bd43d9caa6e02c990b0a82652dca
c20ad4d76fe97759aa27a0c99bff6710
c51ce410c124a10e0db5e4b97fc2af39
98f13708210194c475687be6106a3b84
3c59dc048e8850243be8079a5c74d079
b6d767d2f8ed5d21a44b0e5886680cb9
37693cfc748049e45d87b8c7d8b9aacd
34173cb38f07f89ddbebc2ac9128303f
c16a5320fa475530d9583c34fd356ef5
6364d3f0f495b6ab9dcf8d3b5c6e0b01
182be0c5cdcd5072bb1864cdee4d3d6e
",
        );
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test2.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha1_hash() {
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
            &Arc::clone(&file),
            "sha1",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test3.txt").unwrap();
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
        std::fs::remove_file("test3.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha224_hash() {
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
            &Arc::clone(&file),
            "sha224",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test4.txt").unwrap();
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
        std::fs::remove_file("test4.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha256_hash() {
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
            &Arc::clone(&file),
            "sha256",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test5.txt").unwrap();
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
        std::fs::remove_file("test5.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha384_hash() {
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
            &Arc::clone(&file),
            "sha384",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test6.txt").unwrap();
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
        std::fs::remove_file("test6.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha512_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test7.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "sha512",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test7.txt").unwrap();
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
        std::fs::remove_file("test7.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha3_224_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test8.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "sha3-224",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test8.txt").unwrap();
        let expected_content: String = String::from(
            "b0ce995802adb7b186a38747c99bdded7266f94f38ce3fa7c7e504ca
0002dbddde7f0709b3eee2029fae49c4339871636c63fcfabb5e7800
1f4dfbc64a778819a7a2955c45ced3e8febb70bc0549af7d780ccc31
06690a656afc21aece665bf081e1aad83f3b8a431805b11179b39566
a73d044ff856fe8cc41281c6623ec693c4ab7864c857218e09f9f876
1e194067a7bb8cc16c465f5e5f762e2b4f925f8cf661353d27e7c65d
95a8f823a2e12c1c9d6be7378ba7bf29aaf9345c4caa20c7405c8464
bc8051b292ac5a79786863b882a757d9614c51c15bb015d411f0819a
94c8e4e2fe572b6ac593aff7d1fd7066e39279ea54101158b8e712cb
b89f89ea282cc149f04623635c24f5a4ca6671200c07673080c9f201
4bc473db4b081c64afc7ef3d05bf11e081a989e4c548d185144dfe82
71a022fc02222d9214aee3641bbfd35a706f3e66975a1f949a80abc3
19383e289c092953eed7996a9d501ce322ecf4a3fb8b2f74259205e1
3a3e7778bf1a23dabc31206762060419d61b4ecb53029bc55c416869
1875656aaf7ea2cbb93d9b2bf21352f6f2edc80e7db9b7436fcfdfe5
0f05e8bdad86e0fd5b21cf878124717b30a3d9765631e689c3d2c067
",
        );
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test8.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha3_256_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test9.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "sha3-256",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test9.txt").unwrap();
        let expected_content: String = String::from(
            "2e16aab483cb95577c50d38c8d0d7040f4672683238446c990babbca5ae133c8
b6636575af0a6514c0efcb59bd33fb88e6d8fc64c3ae634cf77fef0da87e95c9
aed00f32306a486cd96be5cd2b5f054bcb58cc23dce3c82a3d5af1c631c1265b
18fdd8e98e1eaab5e303828fbd7fabc51e02b75d805380689a80ad9f86974405
dd121e36961a04627eacff629765dd3528471ed745c1e32222db4a8a5f3421c4
4410fc15c5a3cde7a2b5366a41dbc95e6547a6021efdff98cfcd5e875e8c3c70
1a9a118cb653759c3fcb3bd5060e6f9910c8c27008dd11fe4315f4635c9caa98
1ad7a51ebb6db8cfd0f40d83e398f0a8ad6e7fd4b98e6623b92cfc7c18c4325a
f4e39327cb811e8ea6ae4c9e5fa9ca7a8bfb16e5bd8d89d1a7c7cbd80190ad61
7faeca1b13d1a7f909f21989ff203fa36abb4be730336186f9c49c6e56a890c0
7b0155cb3acfe3a85ad60bcc83fecfc4e1d8e02077c5381f38f005b653ac4d18
39604bdfa135910de937cd3ca347347a1e22c735877c21591d29fe8d2b5844f7
b9056d797418ac18bd541104a6fc472414a348be3a214c0bd93c577f03e3f169
74733b5d1ec0c5e611cc68ab4c656cee5c5241bb09012c73ff5f9a02077c8532
d5801fd41203eeab32fb40335c47fce04361491b37c430b186035ef61dc3ad9a
f372fd5a0bce0ade7c2339a622d124fd1950a9bc0584611f8c334931e39ced32
",
        );
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test9.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha3_384_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test10.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "sha3-384",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test10.txt").unwrap();
        let expected_content: String = String::from("fdad4aa0d9cf5e9904e5e252d24b7969f36f517fb2196d1844a4a3fd6592fbe3536e7d13dbfe6e79a923bd301cde8382
cf989f20667b39d50bf5865017a9d50c0b3c8cc2c7129053d07fb0d4ce7e17f5c3904456d57a31eb0158885edc2c8281
8983f8e1f76e7a25ebac4fcb8c74745ab572338cd684bde01fedbeef456ce88d34a3906b4f9648a56398d40e2d960c92
d691f8164cbb77bc21934dafa6531c62f60ebd0056d0b3310d32deafa822da5122f9aeea7300ca5f2f823f7106bca248
48cbe0a67ec78f9e5313b88de9cff586b270e399b52d64b5226c87fc4cd31e986a3f21b63e9135404ceadfc1199e993e
2fc3117e9fee4702fca8abe0d0edec789b502dc97a8c5eb4023e923f4cfc95163cd32df28d5daff90d30980343e26979
8ad2282a10c5690bf8d59dadd7dcf08a42e3ae6339548848af4a9dcd274fe5c023243eb34a2dfbe0ec0a13ab8df2a06c
d9aec7dbbea064aedc2d2d3793bcc76d42a137a1284d7c19504a2078d178b1fbd77a5a7a29f2e342ef2045fe54cb796e
3deae2ace8609bfae36883d9aba4c90b4fd86500be9a1e777ee7456e684e13a0e3875e2387facb723327a4282fe2b602
bb6163e8d76c6f4096ccb7f1b8b12a73a78aa415994a0476535c0e29a9e1cbc3202ccfad72c1ee3a87245cf7d5830ef4
eb66fdbedc8e01a2041799bd61ab44a0125220937786d030d84f0de48d59fcbe901942fb7eb17d451560c32097775afc
1618c8e3044a1d03b8ad0088efca5cfcd8b30fc99e5c8fb7ef1fef368c196d2f14fcec4a5ef074b0d7d145d98573e6cd
be2b1047db1c44b56dd4a70e3050707f7638f6df165850bfc68f5511a550b4c4005cafd192a80438fe334e7f88d5ebea
cf7df926fefb5194a36d00ec8fb025f3ed242f4233100a0a470468b47c1064edb49e78634d4bb77c69df864f12fa9655
c7441fbb2b270adfff0ef242adc120ed41cc7d3ad0550555dd0d19bd2caa8b87dcae4ebfdf14dd0676497f9a54dfdb6e
8b6a14a1f14549c0e67fa20d05303e34343decf031563a9c8e66085f35c08fc232be7b9ec3410b43b373d4be8981a925
");
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test10.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_sha3_512_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test11.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "sha3-512",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test11.txt").unwrap();
        let expected_content: String = String::from("742b02a28f25e48581a12bd7b6cf6bde3ac9cd3c557615501d136678e7337847e951b94587f45927080fcc6d465f236f6a6dac3049348af9e7677a0404aa2f1a
fa292e9d4b0c7929ba4a332f6c8c05f5451ba15c9d2930de63f2e1de680060208191a8f41af6b676c31ecfc970e6c444e9ea36b780b45763c39370a12ee7749c
96ee504d72084410ab6ea61f1e670c32d43622a1a158dfc682928b948fb24f23124191618ea3a5226b9df295d81b1526a4d054b1be49c8bba1461e863275cd9c
4669d32941f726f318bde37a8626aaf4b16e7defec3ed689daf41caa8176678ba1486008b6797fff88dae676cffdc4ae87128a200498a830c3b6b3286fe0bbf6
0af1abec626b095704a5b03c13e47c3c18bcedb78566b6cadc4d5201cdb27691ce62fe60835587d41c8290616ad4ff1018b14dac6f83ff005922b25925fa4e6a
0a14418e93312aa65ac9e970d4278a769c842a2019a094ceb111380073d19bc32e7b599768a45386943ebc7f2eab4893c01330f208aa20c6dbe47b2b183fdc67
f235c129089233ce3c9c85f1d1554b9cb21952b27e0765bcbcf75d550dd4d2874e546889da5c44db9c066e05e268f4742d672889ff62fb9cb18a3d1b57f00658
bff373da29b813caa0465f6ee58b77ee4ebfe66b00b0b2517767cff0cd8e657376f9a066f31e18f6dbe08754d213451b0e14426c3b950e32fac3c90d81967bd7
432243416a9ad2a62f96237defbd2767e8925125f9b764665854d1568401fd1192026a1976570e3873d8063c702817a4647ab5533457411db0238350b6767e61
edaaf18ba66f4ef92d09e386a095d37dc41ab51b10b99b1b45837b3af66723adcbf61157c0a5730284fd0a60d2472f554c90172d6d500940aee92525d627acbf
53c08c5e90af9bfa55d77bd92d4d23ad83ee80b83366f15701ca4463a421528e01b2835c0d82d9f73427631ba72a92722e17041f8dbf94e8c3f8eda316cff2f5
4f1466999e95b9767883209830ab4e0ab1cf70cd0fc8d18a24ee45ebf9c9cfe691808dcfe3fc1b2efe557a243303960c73f9825ad72f85a3312271b3fd64f7b6
a114cd1299fea4e32d86d9bc877dddf99dee3bf306477e744e894d25a81c713c7d7109883779e6f36ed3383f50adf107b757c441fbbdd0219e59aed672d2cb41
1cf3742cba3799b32f1469483ad7cfa01cf1e977aad0baaa9fde939d07e0ecdd02a0d0bfffc408d2f27985d246a454f024ccfa5072f764218285fb7fe83bc9f9
653a7d9cb4354cd9209858dab9d03e23d210713ec18d8992c8761659d37f056455be908ac4a51cfebc56c555f15a282816c2bd48336e739ed38b94d384bd2c4c
1f3c353f436a5df34ae32e889ad6c8938909a7beeff1803c922af1d5a466dfd5add2eeb284a7190d68f6b327c04e6e105b2bc570e998fdeb4b57eff890f4e49c
");
        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test11.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_blake2b_512_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test12.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "blake2b-512",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test12.txt").unwrap();
        let expected_content: String = String::from("7a4754f2de4589268c2a2d11914f71a1cf170c3f245e9b0593c27ab27f8d19a962da68e8d7e2d229e52510481ef285a0031ad3ac88f35a586ab6347b1716db02
dba297d408c4c0a2ddf1cfbcc6696b440d35738168888c79ae4704e4a87d76f9b913552d15a93bbc2f15ba600d54d5a16a383c98972349661bb5dfa4c500b349
6008769395ad460cb7ebdf92bc333eec778436d7772bcbc9d783ffcf9647eabe7c41fcdc1eeafae541a1f79b0e9bfe47b90415c7251c6269f30bb712fac51c8d
0e86396455477a8a17e01419a5212f30858b4a5bde5edb8d8619c7b5fd39a5cc4710d0c44823f49e9f267fa8b478a598b9c330e47c6769f99021db7c4ce7de51
f2e74cbc3eff574bbc45333c30edb947858543afda4cafdde2903324c9de0bd908b00575c556bd7b8aa2e32a32598a4d5f95cd4490b60a567a3d53680a3310f2
7aa7e388f8145d395ac616bb526eaa35b10069f49e2b36d7327157d1d4af360dfbbfea805aa7e405ed025ce5eadd56c27c40b92991727a5a16b51df5604ad006
b7a5a0f0fb0c4a128b8a3e042fc860775d68d825bb3bf180479d0e12b1884e2652fe51ddb9c991b73824fc15609d82cb1cc19053db7dc7637288091f6027bbce
1da541ba91a8560c5dd0c1a4adc836dc4ac96bf5c407a89edb0a49d46de058a713c7b3d3fc8e0324f602c3a41978ef01dccb989eed22aa65bddc5621765713d3
92ce61bf50a5c299bc88d6adad5db7b68c4b61abb7760947e8b9898c99312b18ba974d427e1699ede1be7c1c25b03440235a41a71ab2b4d1410399b72da87111
8c715c0b894785852fbc391d662e2131bf0f0c703852f25b1c07429f35dc67ec8df5998acd4cafd4f1ff7019ebfda0877f79d6b91c1b98084efbb7314258608c
3733d5bf4f3d2608ba160adf4a8cddbf545f77b417e3ee3a9e5d3b0afb351579125db853e5bce15d5e82c723f29de1ef294341f0ca3e8b3d3431cec7ac316f34
08949f758439c6293fe5924defaf3e32bb79b9a93c1331f019c51b386557a9412b27f5a60a80bfa1f524c0d0c2e1f63c5b93d108a9a3af8cdb7fc87c765fca3f
78a42a89d6e562a77f7e1782f29ac05e8c7dea6ccf51c50c9e38bbf09611577f4f6743ef6ea0419c9d536843a42df0195fbc2a404aebfe77de8edc82eb858694
931314c2ee664b02b8902abbf64ae403208d939152c618a840aba58a3042cab624f16ba2b4e59f49a651d234527e62d06745bd7fff6707b1692e534e8dcfaf56
33ddec12969ebc28a9a8512e4814170f421e97289c615dc7b1e117eb23d5b6523d4682f641871f9161966579b3714d15e956079a6cd07b4a0ea8b5631674f27a
afd85ebe3a733ccf64c11c681e05595490854c4a485be585dfff1f2c1958823a0fa11069467c82602b3fece173df052206012b8df3a90b68569c32b3b85c817c
");

        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test12.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_blake2s_256_hash() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test13.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "blake2s-256",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test13.txt").unwrap();
        let expected_content: String = String::from(
            "3cb66864b2523e549ae456039edcd9156e657edddfebff37e80fd1a80fc38863
57e36dae300302953c953b59a1b263cb314326db44e919ca4acd57e1da8f0543
f309b7dd406f649ea531807938154d576331c227d40501a8399fbce9a0b3175f
b94039da1e9c82d9578a00af4535dd697687794e6f757c300603915426162a1b
f75ef2a61a17d7acc2561bac08cb7dd4d4654ca4d6311bb7865bc0757873b0ba
92a30ce2343131f27bdc139b6d337393a9b8549ce6b1a203da96417f5d1864a0
109a7947e24479dd8391df82eb9af87d135de808edda86e5b9c0f78d7d05b170
bf652f9276071fddbd19bc05b139d687dc248257a5c28b2957ed04c1a55f4e40
584de556904b549357e11090ad216e56b5f879d95d0bcdeaf2d2b065a8bce6a7
6ef2938d68ed56f7bada1d3055b6b591492a73fddeac860ee2d01f9c3f607d9b
6f14e569ae0ca8f16d019169306bf77382ca79442b36fc7fb5bfbaa27d132bae
c974d441f2d4398b0f1e1f2cdcd0ad1181773a2f7108bb75649538c24fd89b20
803d729a8ca8142c2d1a952c969a846c07d0c35b1b9893d2ee54e9235b9800c5
9dd3dfa071597218fc179995378cc1eb9a0d5da45afba5b86a808241894e91a1
ad2e6c09da74d174a007c1eac6797f1b2f2ce21fa4e560e92902e0422bd9fa55
fd5f8f39e9a41e624fb837cd766f55805bd120aac42fe4bc426497eeb7aeaacf
",
        );

        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test13.txt").unwrap();
    }

    #[test]
    fn test_generate_wordlist_part_with_whirlpool() {
        let nb_of_passwords: u64 = 16;
        let dict_indexes: Vec<usize> = vec![0, 0];
        let formated_mask: Vec<char> = vec!['\0', '\0'];
        let mask_indexes: Vec<usize> = vec![0, 1];
        let dict: Vec<u8> = vec![b'0', b'1', b'2', b'3'];
        let file: Arc<Mutex<File>> = Arc::new(Mutex::new(File::create("test14.txt").unwrap()));
        let result: Result<(), WorgenXError> = generate_wordlist_part(
            nb_of_passwords,
            dict_indexes,
            &formated_mask,
            &mask_indexes,
            &dict,
            &Arc::clone(&file),
            "whirlpool",
        );
        assert!(result.is_ok());

        let content: String = std::fs::read_to_string("test14.txt").unwrap();
        let expected_content: String = String::from("e823fde96090bd56b012b6f2e152cc87d89b7fa0a1c8688579c0be2423ba3c16ea81f8b67e9346c3c03e59121bcfd2dd8c753beb20e8ad4e95870f2039206bc1
4585dec207baf8ece36ee2a871f9a5dfb2df3039de04c456db019d74e736ed742c4a8a3f7f094bab2364577a274159bc13edb7b20385bd44fafc57afd214940f
04f6127b0623ef98c4710b6c087f27e3abbe16c4250f1109b0659c2b742138bb976451e3124604624504ec6a5f4469c246aedce163be86641f5a25363cba0588
180fadc681164f5d3a29ea42c49c7bb9352559652505b1eb9e16f029b5070781163cfd8836cd7e8eed72b2122a70adb7d9e0a2fff0cb0f0958b3a45147092a8f
cd977ca1c2ffee1fcc1c4cb64d52412d61cdc81ad01ae93148b9a2c7e940da8906048743b17023f04463a468aea5965ce47daef5b9f1f4901f2c2b540687a4d0
ebdaf73bf4ba32150a6f9c7cde8c7e6ea327324a9445407d727bd0a97fbd77c6d456d1a8942096e596238df0a754de10c1309a918ed0d1203cc379b24f062224
24e3253ceeb4e32b854c86dafd7ddd6d747d8c9de574a003e9d5a590cc20e1254b853a85ae845ab1266874bb70da8dac00ca3991c2f3e46e008ad19340e06dbf
72135fa8697eae1c9002bad98bc07d44abc9180794dfa54ed68a1a23d750e505426b9ed67510e16f778e3cecf74770692498d6b7d964856f041b7cc7c1c96a9d
5bb1d9d5be046db9d749d4351e92ff8d53b1a03a20f4eb073f38331e0a8e73d2d08e5af1008cda104b20a6a7ae6bf4de22c96c3d81a8c7a0521ba5f0767d8389
75b5bd3160157e9ea5576f9bf9e60f3d680cc77b0f9f4fae492c0c0917a6995c860c985c99b87eae874e35f2b38a06efaa3bda5f28cfab942dbc34886db64f4d
c3e929030d6d1dfc8332c1f72ef49e379d124c2a583ca723f526e51321882de03a4f0a85558cd73eab6b7afe69773243b19aba674157b527b472791a4d2bd37e
18417525e4d773854fdf954b1c44810628a2c67ea3b3f64229858721a614683a4c125aa5e7ba1fd7504c4a8e654239666eab6a7d2e67c4f837b1e12459ca2680
665788107ae4bdcb0c28a7ce1c64fda77a676cbe73ef8b99e2621a675a58c7f84f907f2909512782a7df9736254f19a8560cbbc4d0d938ff3ad43435d77656a1
5350b5db7c67126b3c910cbf2044416bca5aa4495fe37451bd0ae35f841dc33a7e3a09596b95fe1fcaed190a232756b082652e4f0c0fcf333c1990ba05c7febf
adc14b8a3d1c67bb3bcf764d7e11dc0d53474221f9c30a61b344bb55c834eea6f1f2c7e5bffc677848291e759df5a83fbeee3d7d05bfbce0d4fff52c32882260
56fcc26b6aacefd921d97fdabe8d927a6f5dd471a472aa546d4863552b728954c5e5d47ceae1ad9a012fecf148def311b957d7251000646ae590ff0484b03401
");

        assert_eq!(content.lines().count(), 16);
        assert_eq!(content, expected_content);
        std::fs::remove_file("test14.txt").unwrap();
    }
}
