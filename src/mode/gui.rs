// Internal crates
use crate::password;
use crate::system;

pub fn run(){
   loop {
        print_menu();
        let choice = system::get_user_choice();
        match &*choice {
            "0" => break,
            // "1" => generate_wordlist(),
            "2" => password::main_passwd_generation(),
            // "3" => benchmark_cpu(),
            _ => (),
        }
    }
    println!("Bye!"); 
}

/// This function is charged to display the header menu
///
/// # Example
/// ```
/// display_title();
/// ```
/// 
fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

/// This function is charged to display the menu
///
/// # Example
/// ```
/// print_menu();
/// ```
/// 
fn print_menu() {
    display_title();
    println!("\n   WorgenX by Xen0rInspire \n");
    display_title();

    print!("\n\n");
    println!("1 : Create wordlist(s)");
    println!("2 : Generate random password(s)");
    println!("3 : Benchmark CPU");
    println!("0 : Exit WorgenX");
}
