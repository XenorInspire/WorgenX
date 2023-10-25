use std::{
    fs::OpenOptions,
    io::Write,
    thread::sleep,
    time::{Duration, Instant},
};

use std::thread::Thread;

/// The default dictionaries used to generate the password(s)
pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const NUMBERS: &[u8] = b"0123456789";
pub const SPECIAL_CHARACTERS: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]_{|}";

fn main() {
    let start = Instant::now();
    let mask = "a&X;\\\\\\?????15?!/";
    // Create an array with all charachters from the constants
    let mut dict: Vec<u8> = Vec::new();
    // dict.extend(LOWERCASE);
    // dict.extend(UPPERCASE);
    dict.extend(NUMBERS);
    // dict.extend(SPECIAL_CHARACTERS);

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

    // nb of possibilities = pow(dict.len(), nb of '?')
    let nb_of_passwd = dict.len().pow(mask_indexes.len() as u32);
    sleep(Duration::from_secs(2));

    let num_cpus = num_cpus::get(); // number of cores
    let mut threads: Vec<Thread> = Vec::new();
    let mut nb_of_passwd_per_thread = nb_of_passwd / num_cpus;
    let mut nb_of_passwd_last_thread = nb_of_passwd_per_thread + nb_of_passwd % num_cpus;

    // Divide the work between the threads in order to have the same amount of work for each thread
    // Divide dict_indexes in num_cpus parts

    let mut dict_indexes_per_thread: Vec<Vec<usize>> = Vec::new();
    let mut start_idx = 0;
    let mut end_idx = 0;
    if num_cpus > 1 {
        for i in 0..num_cpus {

        }
    } else {
        dict_indexes_per_thread.push(vec![0; mask_indexes.len()]);
    }
    
    for dict in dict_indexes_per_thread {
        println!("Dict : \t\t{:?}", dict);
    }

    // for _ in 0..nb_of_passwd {
    //     let mut line = String::new();
    //     (0..final_mask.len()).for_each(|i| {
    //         let mut found = false;
    //         for idx in 0..mask_indexes.len() {
    //             if i == mask_indexes[idx] {
    //                 found = true;
    //                 line.push(dict[dict_indexes[idx]] as char);
    //                 break;
    //             }
    //         }

    //         if !found {
    //             line.push(final_mask[i])
    //         }
    //     });
    //     for idx in (0..dict_indexes.len()).rev() {
    //         if dict_indexes[idx] < dict.len() - 1 {
    //             dict_indexes[idx] += 1;
    //             break;
    //         } else {
    //             dict_indexes[idx] = 0;
    //         }
    //     }
    //     // sleep(Duration::from_millis(200));
    //     // println!("Dict indexes : \t{:?}", dict_indexes);
    //     save_passwords("passwords.txt".to_string(), line);
    // }

    println!("Number of cores : {}", num_cpus);

    println!("Time elapsed is: {:?}", start.elapsed());
}

pub fn save_passwords(file_path: String, password: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)
        .unwrap();
    file.write_all(format!("{}\n", password).as_bytes())
        .unwrap();
}
