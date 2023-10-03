use std::time::Instant;

/// The default dictionaries used to generate the password(s)
pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const NUMBERS: &[u8] = b"0123456789";
pub const SPECIAL_CHARACTERS: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]_{|}";

fn main() {
    let start = Instant::now();
    let mask = "a?b??????15?!/";
    // Create an array with all charachters from the constants
    let mut dict: Vec<u8> = Vec::new();
    dict.extend(LOWERCASE);
    dict.extend(UPPERCASE);
    dict.extend(NUMBERS);
    dict.extend(SPECIAL_CHARACTERS);

    // Create an array with all the indexes of the mask
    let mut mask_indexes: Vec<usize> = Vec::new();
    for (i, c) in mask.chars().enumerate() {
        if c == '?' {
            mask_indexes.push(i);
        }
    }
    println!("Mask indexes : {:?}", mask_indexes);

    let num_cpus = num_cpus::get(); // number of cores
    // Give the time of the program execution
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    

}
