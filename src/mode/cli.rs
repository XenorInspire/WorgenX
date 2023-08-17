/// This function is charged to schedule in CLI mode the execution of the different features of the program
/// according to the user's choices
/// 
pub fn run() {
    let args = std::env::args().collect::<Vec<String>>();
    display_help();
}

/// This function is charged to display the help message with all the features of the program
/// 
/// # Example
/// ```
/// display_help();
/// ```
/// 
fn display_help(){
    println!("Usage: worgenx_cli <command> [options]");
    println!("Commands:");
    println!("  -d, --dict\t\tGenerate a wordlist");
    println!("  -p, --passwd\t\tGenerate random password(s)");
    println!("  -b, --benchmark\tBenchmark CPU");
    println!("  -v, --version\t\tDisplay the version of WorgenX");
    println!("  -h, --help\t\tDisplay this help message\n\n");
    println!("Below are the options for the main features:\n");
    println!("  --- Dictionary generation ---");
    println!("  You must specify at least one of the following options: -l, -u, -n, -s");
    println!("    -l, --lowercase\t\t\tAdd lowercase characters to the words");
    println!("    -u, --uppercase\t\t\tAdd uppercase characters to the words");
    println!("    -n, --numbers\t\t\tAdd numbers to the words");
    println!("    -x, --special-characters\t\tAdd special characters to the words");
    println!("\n  This parameter is mandatory:");
    println!("    -s <size>, --size <size>\t\tSize of the words");
    println!("\n  The following options are optional:");
    println!("    -o <path>, --output <path>\t\tSave the wordlist in a file");
    println!("    -d, --disable-loading-bar\t\tDisable the loading bar when generating the wordlist");
    println!("    -j, --json\t\t\t\tOutput in JSON format, it automatically disables the loading bar\n\t\t\t\t\tCombine with -o to save the json output in a file");
    println!("\n  --- Password generation ---");
    println!("  You must specify at least one of the following options: -l, -u, -n, -s");
    println!("    -l, --lowercase\t\t\tAdd lowercase characters to the words");
    println!("    -u, --uppercase\t\t\tAdd uppercase characters to the words");
    println!("    -n, --numbers\t\t\tAdd numbers to the words");
    println!("    -x, --special-characters\t\tAdd special characters to the words");
    println!("\n  These parameters are mandatory:");
    println!("    -s <size>, --size <size>\t\tSize of the passwords");
    println!("    -c <count>, --count <count>\t\tNumber of passwords to generate");
    println!("\n  The following options are optional:");
    println!("    -o <path>, --output <path>\t\tSave the wordlist in a file");
    println!("    -j, --json\t\t\t\tOutput in JSON format\n\t\t\t\t\tCombine with -o to save the json output in a file");
    
}