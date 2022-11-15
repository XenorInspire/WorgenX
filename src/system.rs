// Extern crates
use std::fs::File;
use std::io::Write;

// This function is charged to save the password in a file
fn save_password(password: &str, file_name: &str) {
    let mut file = File::create(file_name).expect("Unable to create file");
    file.write_all(password.as_bytes())
        .expect("Unable to write data");
}
