// Internal crates
use crate::error::{ArgError, WorgenXError};
use crate::password::{self, PasswordConfig};

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
            if args.len() < 3 {
                return Err(WorgenXError::ArgError(ArgError::MissingConfiguration(
                    args[1].clone(),
                )));
            }
            match allocate_passwd_config_cli(args) {
                Ok(password_config) => {
                    password::generate_random_passwords(&password_config);
                }
                Err(e) => {
                    return Err(WorgenXError::ArgError(e));
                }
            }
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
/// This function is called only if the user specifies the -p or --passwd argument
/// It returns an ArgError if the syntax is incorrect
/// It returns Ok(()) if the syntax is correct
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Return
///
/// * `Result<(), ArgError>` - An ArgError if the syntax is incorrect (or invalid config/values), Ok(PasswordConfig) if the syntax is correct
///
/// # Example
/// ```
/// let args = std::env::args().collect::<Vec<String>>();
/// let result = check_passwd_config_cli(args);
/// ```
///
fn allocate_passwd_config_cli(args: Vec<String>) -> Result<PasswordConfig, ArgError> {
    let mut numbers = false;
    let mut special_characters = false;
    let mut uppercase = false;
    let mut lowercase = false;
    let mut length = 0;
    let mut number_of_passwords = 0;
    let mut output_file = String::new();
    let mut json = false;

    for i in 2..args.len() {
        match args[i].as_str() {
            "-l" | "--lowercase" => {
                lowercase = true;
            }
            "-u" | "--uppercase" => {
                uppercase = true;
            }
            "-n" | "--numbers" => {
                numbers = true;
            }
            "-x" | "--special-characters" => {
                special_characters = true;
            }
            "-s" | "--size" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(value) => {
                            if value == 0 {
                                return Err(ArgError::InvalidNumericalValue(
                                    args[i + 1].clone(),
                                    args[i].clone(),
                                ));
                            }
                            length = value;
                        }
                        Err(_) => {
                            return Err(ArgError::InvalidNumericalValue(
                                args[i + 1].clone(),
                                args[i].clone(),
                            ));
                        }
                    }
                } else {
                    return Err(ArgError::MissingArgument(args[i].clone()));
                }
            }
            "-c" | "--count" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(value) => {
                            number_of_passwords = value;
                        }
                        Err(_) => {
                            return Err(ArgError::InvalidNumericalValue(
                                args[i + 1].clone(),
                                args[i].clone(),
                            ));
                        }
                    }
                } else {
                    return Err(ArgError::MissingArgument(args[i].clone()));
                }
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                } else {
                    return Err(ArgError::MissingArgument(args[i].clone()));
                }
            }
            "-j" | "--json" => {
                json = true;
            }
            _ => {
                return Err(ArgError::UnknownArgument(args[i].clone()));
            }
        }
    }

    if !lowercase && !uppercase && !numbers && !special_characters {
        return Err(ArgError::MissingConfiguration(args[1].clone()));
    }

    Ok(PasswordConfig {
        numbers,
        special_characters,
        uppercase,
        lowercase,
        length,
        number_of_passwords,
        output_file,
        json,
    })
}

/// This function is charged to check the syntax of the arguments passed to the program
/// It does not check the values of the arguments
/// This function is called only if the user specifies the -d or --dict argument
/// It returns an ArgError if the syntax is incorrect
/// It returns Ok(()) if the syntax is correct
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Example
/// ```
/// allocate_wordlist_config_cli(args);
/// ```
///
fn allocate_wordlist_config_cli(args: Vec<String>) -> Result<(), ArgError> {
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
