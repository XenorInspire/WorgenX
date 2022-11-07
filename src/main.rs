use std::io;

fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

fn print_menu() {
    let mut choice = String::from("");
    while !choice.eq("0") {
        display_title();
        println!("\n   WorgenX by Xen0rInspire \n");
        display_title();

        print!("\n");
        println!("\n1 : Create wordlist(s)");
        println!("2 : Generate random password(s)");
        println!("3 : Hash plaintext");
        println!("4 : Benchmark CPU");
        println!("0 : Exit WorgenX");

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Failed to read line from stdin");
        choice = buffer.trim().to_string();

        match &*choice {
            // '1' => create_wordlist(),
            // '2' => generate_password(),
            // '3' => hash_plaintext(),
            // '4' => benchmark_cpu(),
            _ => (),
        }
    }
}

fn main() {
    print_menu();
    println!("Bye!");
}
