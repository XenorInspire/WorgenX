struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
    pub content: String,
}