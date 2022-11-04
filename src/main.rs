fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

fn print_menu() {
    let mut choice: char = ' ';
    while choice != '0' {
        display_title();
        println!("\n   WorgenX by Xen0rInspire \n");
        display_title();

        print!("\n");
        println!("\n1 : Create wordlist(s)");
        println!("2 : Generate random password(s)");
        println!("3 : Hash plaintext");
        println!("4 : Benchmark CPU");
        println!("5 : Exit WorgenX");
        choice = '0';
        // match choice {
        //     '1' => create_wordlist(),
        //     '2' => generate_password(),
        //     '3' => hash_plaintext(),
        //     '4' => benchmark_cpu(),
        //     '5' => exit(),
        //     _ => println!("Try again!"),
    }
}

fn main() {
    print_menu();
    println!("Bye!");
}
