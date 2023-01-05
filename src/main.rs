// Internal crates
mod password;
mod system;
mod dict;

// This function is charged to display the header menu
fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

// This functrion is charged to display the menu
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

// This function is the "entry point" of the program
fn main() {
    let mut choice = String::from("");
    while !choice.eq("0") {
        print_menu();
        choice = system::get_user_choice();

        match &*choice {
            // "1" => generate_wordlist(),
            "2" => password::main_passwd_generation(),
            // "3" => benchmark_cpu(),
            _ => (),
        }
    }
    println!("Bye!");
}
