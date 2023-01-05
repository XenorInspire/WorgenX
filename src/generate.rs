// Internal crates
use crate::system;

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