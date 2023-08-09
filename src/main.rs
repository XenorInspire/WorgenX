/// This constant is charged to store the OS name
#[cfg(target_os = "windows")]
pub const OS: &str = "windows";

#[cfg(target_os = "linux")]
pub const OS: &str = "linux";

#[cfg(target_os = "macos")]
pub const OS: &str = "macos";

#[cfg(all(not(feature = "gui"), not(feature = "cli")))]
compile_error!("You must specify a mode: 'gui' or 'cli'.");


/// This constant is charged to store the program mode (GUI or CLI)
#[cfg(feature = "gui")]
mod mode {
    pub mod gui;
    pub const MODE: &str = "GUI";
}

#[cfg(feature = "cli")]
mod mode {
    pub mod cli;
    pub const MODE: &str = "CLI";
}

// Internal modules
mod dict;
mod error;
mod generator;
mod password;
mod system;

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

/// This function is the "entry point" of the program
/// 
fn main() {
    println!("{}", mode::MODE);
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
