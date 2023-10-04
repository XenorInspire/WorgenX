use std::time::Instant;

/// The default dictionaries used to generate the password(s)
pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const NUMBERS: &[u8] = b"0123456789";
pub const SPECIAL_CHARACTERS: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]_{|}";

fn main() {
    let start = Instant::now();
    let mask = "a?X?\\\\\\?????15?!/";
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
    for (i, c) in mask.chars().enumerate() {
        match c {
            '\\' => {
                if escaped {
                    escaped = false;
                    final_mask.push(c);
                    continue;
                } else {
                    escaped = true;
                    continue;
                }
            }
            '?' => {
                if escaped {
                    escaped = false;
                    final_mask.push(c);
                    continue;
                } else {
                    mask_indexes.push(i);
                    final_mask.push(0 as char);
                }
            }
            _ => {
                final_mask.push(c);
                continue;
            }
        }
    }

    println!(
        "Mask : \t\t{:?}",
        mask.to_string().chars().collect::<Vec<char>>()
    );
    println!("Final Mask : \t{:?}", final_mask);
    println!("Mask indexes : \t{:?}", mask_indexes);

    let num_cpus = num_cpus::get(); // number of cores
                                    // Give the time of the program execution
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
}
