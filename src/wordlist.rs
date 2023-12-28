// Internal crates
use crate::dict;

// This struct is built from the user's choices will be used to generate the wordlist
pub struct WordlistValues {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub mask: String,
}

// This struct is built from the WordlistConfig struct and will be used to generate the wordlist
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
/// * `wordlist_config` - The struct containing the user's choices
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
/// A tuple containing the vector of char (final_mask) and the vector of indexes (mask_indexes)
///
fn format_mask_to_indexes(mask: &str) -> (Vec<char>, Vec<usize>) {
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

    (final_mask, mask_indexes)
}

/// This function is charged to build the WordlistConfig with the settings of the wordlist
///
/// # Arguments
///
/// * `wordlist_values` - The struct containing the user's choices
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
