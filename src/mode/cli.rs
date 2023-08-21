// Internal crates
use crate::error::{ArgError, WorgenXError};

/// This function is charged to schedule in CLI mode the execution of the different features of the program
/// according to the user's choices
///
pub fn run() -> Result<(), WorgenXError> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(WorgenXError::ArgError(ArgError::NoArgument));
    }
    match args[1].as_str() {
        "-d" | "--dict" => {
            println!("Not implemented yet");
        }
        "-p" | "--passwd" => {
            println!("Not implemented yet");
        }
        "-b" | "--benchmark" => {
            println!("Not implemented yet");
        }
        "-v" | "--version" => {
            println!("WorgenX v{}", env!("CARGO_PKG_VERSION"));
        }
        "-h" | "--help" => {
            display_help();
        }
        _ => {
            return Err(WorgenXError::ArgError(ArgError::UnknownArgument(
                args[1].clone(),
            )))
        }
    }
    Ok(())
}

/// This function is charged to check the syntax of the arguments passed to the program
/// It does not check the values of the arguments
/// This function is called only if the user specifies the -d / --dict argument or -p / --passwd argument
/// It returns an ArgError if the syntax is incorrect
/// It returns Ok(()) if the syntax is correct
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Example
/// ```
/// check_global_syntax(args);
/// ```
///
fn check_global_syntax(args: Vec<String>) -> Result<(), ArgError> {
    Ok(())
}

/// This function is charged to display the help message with all the features of the program
///
/// # Example
/// ```
/// display_help();
/// ```
///
fn display_help() {
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
    println!("    -o <path>, --output <path>\t\tSave the wordlist in a file");
    println!("\n  The following options are optional:");
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
