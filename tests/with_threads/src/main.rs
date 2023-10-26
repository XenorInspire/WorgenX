use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

/// The default dictionaries used to generate the password(s)
pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const NUMBERS: &[u8] = b"0123456789";
pub const SPECIAL_CHARACTERS: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]_{|}";

fn main() {
    let start = Instant::now();
    let mask = "a&X;\\\\\\?????15?!/";
    // let mask = "a&X;\\\\\\???15!/";
    // Create an array with all charachters from the constants
    let mut dict: Vec<u8> = Vec::new();
    dict.extend(LOWERCASE);
    dict.extend(UPPERCASE);
    dict.extend(NUMBERS);
    dict.extend(SPECIAL_CHARACTERS);

    // // Create an array with all the indexes of the mask
    let mut mask_indexes: Vec<usize> = Vec::new();
    let mut final_mask: Vec<char> = Vec::new();
    let mut escaped = false;
    let mut idx_final_mask: usize = 0;
    for (_, c) in mask.chars().enumerate() {
        match c {
            '\\' => {
                if escaped {
                    escaped = false;
                    final_mask.push(c);
                } else {
                    escaped = true;
                    continue;
                }
            }
            '?' => {
                if escaped {
                    escaped = false;
                    final_mask.push(c);
                } else {
                    mask_indexes.push(idx_final_mask);
                    final_mask.push(0 as char);
                }
            }
            _ => {
                final_mask.push(c);
            }
        }
        idx_final_mask += 1;
    }

    println!(
        "Mask : \t\t{:?}",
        mask.to_string().chars().collect::<Vec<char>>()
    );
    println!("Final Mask : \t{:?}", final_mask);
    println!("Mask indexes : \t{:?}", mask_indexes);
    println!("Dict size : \t{}", dict.len());

    // nb of possibilities = pow(dict.len(), nb of '?')
    let nb_of_passwd = dict.len().pow(mask_indexes.len() as u32);
    println!("Nb of passwd : \t{}", nb_of_passwd);
    let num_cpus = num_cpus::get();
    let dict_indexes: Vec<usize> = vec![0; mask_indexes.len()];

    if num_cpus >= 2 && num_cpus < nb_of_passwd {
        println!("Number of cores : {}", num_cpus);

        let shared_final_mask = Arc::new(final_mask);
        let shared_mask_indexes = Arc::new(mask_indexes);
        let dict_size = dict.len();
        let shared_dict = Arc::new(dict);

        let file = Arc::new(Mutex::new(
            OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open("passwords.txt".to_string())
                .unwrap(),
        ));

        let mut threads = Vec::new();

        let mut nb_of_passwd_per_thread = nb_of_passwd / num_cpus;
        let nb_of_passwd_last_thread = nb_of_passwd_per_thread + nb_of_passwd % num_cpus;
        let mut temp = dict_indexes.clone();

        println!("Nb of passwd per thread : {}", nb_of_passwd_per_thread);
        println!("Nb of passwd last thread : {}", nb_of_passwd_last_thread);

        for i in 0..num_cpus {
            if i == num_cpus - 1 {
                nb_of_passwd_per_thread = nb_of_passwd_last_thread;
            }

            let shared_final_mask = Arc::clone(&shared_final_mask);
            let shared_mask_indexes = Arc::clone(&shared_mask_indexes);
            let shared_dict = Arc::clone(&shared_dict);
            let file = Arc::clone(&file);
            let temp_clone = temp.clone();
            let thread = thread::spawn(move || {
                generate_wordlist_part(
                    nb_of_passwd_per_thread,
                    temp_clone,
                    shared_final_mask,
                    shared_mask_indexes,
                    shared_dict,
                    file,
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
            thread.join().unwrap();
        }
    }
    println!("Time elapsed is: {:?}", start.elapsed());
}

fn generate_wordlist_part(
    nb_of_passwd: usize,
    mut dict_indexes: Vec<usize>,
    final_mask: Arc<Vec<char>>,
    mask_indexes: Arc<Vec<usize>>,
    dict: Arc<Vec<u8>>,
    file: Arc<Mutex<File>>,
) {
    for _ in 0..nb_of_passwd {
        let mut line = String::new();
        (0..final_mask.len()).for_each(|i| {
            let mut found = false;
            for idx in 0..mask_indexes.len() {
                if i == mask_indexes[idx] {
                    found = true;
                    line.push(dict[dict_indexes[idx]] as char);
                    break;
                }
            }

            if !found {
                line.push(final_mask[i])
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

        save_passwords(Arc::clone(&file), line);
    }
}

fn save_passwords(file: Arc<Mutex<File>>, password: String) {
    let mut file = file.lock().unwrap();
    file.write_all(format!("{}\n", password).as_bytes())
        .unwrap();
}
