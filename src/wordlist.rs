// Internal crates
use crate::dict;

// This struct is built from the user's choices will be used to generate the wordlist
pub struct WordlistConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub mask: String,
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
fn create_wordlist_content(wordlist_config: &WordlistConfig) -> Vec<u8> {
    let mut final_dict: Vec<u8> = Vec::new();

    if wordlist_config.uppercase {
        final_dict.extend(dict::UPPERCASE);
    }

    if wordlist_config.lowercase {
        final_dict.extend(dict::LOWERCASE);
    }

    if wordlist_config.numbers {
        final_dict.extend(dict::NUMBERS);
    }

    if wordlist_config.special_characters {
        final_dict.extend(dict::SPECIAL_CHARACTERS);
    }

    final_dict
}
