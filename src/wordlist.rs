// Internal crates

// This struct is built from the user's choices will be used to generate the wordlist
pub struct WordlistConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub mask: String,
}
