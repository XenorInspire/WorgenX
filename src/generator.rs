// Internal crates
use crate::error::WorgenXError;
use crate::system;

// This struct is built from the user's choices will be used to generate the wordlist
struct WordlistConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
    pub wordlist_name: String,
    pub is_mask_mode_enabled: bool,
    pub mask: String,
}

// This is the main function of the wordlist generation module
pub fn main_wordlist_generation() {}

// This function is charged to allocate the wordlist config structure
// fn create_wordlist_config_gui() -> Result<WordlistConfig, WorgenXError> {

// }
