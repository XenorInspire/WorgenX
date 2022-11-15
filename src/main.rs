// Extern crates
use std::io;

// Internal crates
mod system;
mod password;

// This function is charged to display the header menu
fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

// This functrion is charged to display the menu
fn print_menu() {
    let mut choice = String::from("");
    while !choice.eq("0") {
        display_title();
        println!("\n   WorgenX by Xen0rInspire \n");
        display_title();

        print!("\n\n");
        println!("1 : Create wordlist(s)");
        println!("2 : Generate random password(s)");
        println!("3 : Benchmark CPU");
        println!("0 : Exit WorgenX");

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Failed to read line from stdin");
        choice = buffer.trim().to_string();

        match &*choice {
            // '1' => create_wordlist(),
            // '2' => generate_random_password(),
            // '3' => benchmark_cpu(),
            _ => (),
        }
    }
}

// This function is the "entry point" of the program
fn main() {
    print_menu();
    println!("Bye!");
}
