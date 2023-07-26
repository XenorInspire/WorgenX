// Internal modules
mod dict;
mod error;
mod generator;
mod password;
mod system;

#[cfg(target_os = "windows")]
pub const OS: &str = "windows";
#[cfg(target_os = "unix")]
pub const OS: &str = "unix";

/// This function is charged to display the header menu
///
/// # Example
/// ```
/// display_title();
/// ```
fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

/// This functrion is charged to display the menu
///
/// # Example
/// ```
/// print_menu();
/// ```
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

/// This function is the "entry point" of the program
fn main() {
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
