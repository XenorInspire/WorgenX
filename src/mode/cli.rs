// Internal crates
use crate::error::{ArgError, WorgenXError};
use crate::password::{self, PasswordConfig};
use crate::system;

/// This struct built from PasswordConfig and optional arguments will be used to generate the random password
///
struct PasswordGenerationParameters {
    password_config: PasswordConfig,
    json: bool,
    output_file: String,
    no_display: bool,
}

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
                Ok(password_generation_parameters) => {
                    let passwords = password::generate_random_passwords(
                        &password_generation_parameters.password_config,
                    );
                    // TODO: check output_file and json
                    if !password_generation_parameters.no_display {
                        println!("You can find your password(s) below:\n");
                        for password in passwords {
                            println!("{}", password);
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
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
fn allocate_passwd_config_cli(
    args: Vec<String>,
) -> Result<PasswordGenerationParameters, WorgenXError> {
    let mut output_file = String::new();
    let mut json = false;
    let mut no_display = false;
    let mut skip = false;
    let mut one_path = false;
    let mut password_config = PasswordConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
        number_of_passwords: 0,
    };

    for i in 2..args.len() {
        if skip {
            skip = false;
            continue;
        }
        match args[i].as_str() {
            "-l" | "--lowercase" => {
                password_config.lowercase = true;
            }
            "-u" | "--uppercase" => {
                password_config.uppercase = true;
            }
            "-n" | "--numbers" => {
                password_config.numbers = true;
            }
            "-x" | "--special-characters" => {
                password_config.special_characters = true;
            }
            "-s" | "--size" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(value) => {
                            if value == 0 {
                                return Err(WorgenXError::ArgError(
                                    ArgError::InvalidNumericalValue(
                                        args[i + 1].clone(),
                                        args[i].clone(),
                                    ),
                                ));
                            }
                            password_config.length = value;
                        }
                        Err(_) => {
                            return Err(WorgenXError::ArgError(ArgError::InvalidNumericalValue(
                                args[i + 1].clone(),
                                args[i].clone(),
                            )));
                        }
                    }
                } else {
                    return Err(WorgenXError::ArgError(ArgError::MissingValue(
                        args[i].clone(),
                    )));
                }
                skip = true;
                continue;
            }
            "-c" | "--count" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(value) => {
                            if value == 0 {
                                return Err(WorgenXError::ArgError(
                                    ArgError::InvalidNumericalValue(
                                        args[i + 1].clone(),
                                        args[i].clone(),
                                    ),
                                ));
                            }
                            password_config.number_of_passwords = value;
                        }
                        Err(_) => {
                            return Err(WorgenXError::ArgError(ArgError::InvalidNumericalValue(
                                args[i + 1].clone(),
                                args[i].clone(),
                            )));
                        }
                    }
                } else {
                    return Err(WorgenXError::ArgError(ArgError::MissingValue(
                        args[i].clone(),
                    )));
                }
                skip = true;
                continue;
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    if one_path {
                        return Err(WorgenXError::ArgError(ArgError::BothOutputArguments));
                    }
                    output_file = args[i + 1].clone();
                    match system::is_valid_path(&output_file) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(WorgenXError::SystemError(e));
                        }
                    }
                } else {
                    return Err(WorgenXError::ArgError(ArgError::MissingValue(
                        args[i].clone(),
                    )));
                }
                one_path = true;
                skip = true;
                continue;
            }
            "-j" | "--json" => {
                json = true;
            }
            "-O" | "--output-only" => {
                if i + 1 < args.len() {
                    if one_path {
                        return Err(WorgenXError::ArgError(ArgError::BothOutputArguments));
                    }
                    output_file = args[i + 1].clone();
                    match system::is_valid_path(&output_file) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(WorgenXError::SystemError(e));
                        }
                    }
                } else {
                    return Err(WorgenXError::ArgError(ArgError::MissingValue(
                        args[i].clone(),
                    )));
                }

                no_display = true;
                one_path = true;
                skip = true;
                continue;
            }
            _ => {
                return Err(WorgenXError::ArgError(ArgError::UnknownArgument(
                    args[i].clone(),
                )));
            }
        }
    }

    if !password_config.lowercase
        && !password_config.uppercase
        && !password_config.numbers
        && !password_config.special_characters
    {
        return Err(WorgenXError::ArgError(ArgError::MissingConfiguration(
            args[1].clone(),
        )));
    }

    if password_config.length == 0 {
        return Err(WorgenXError::ArgError(ArgError::MissingArgument(
            "-s or --size".to_string(),
        )));
    }

    if password_config.number_of_passwords == 0 {
        return Err(WorgenXError::ArgError(ArgError::MissingArgument(
            "-c or --count".to_string(),
        )));
    }

    Ok(PasswordGenerationParameters {
        password_config,
        json,
        output_file,
        no_display,
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
    println!("    -s <size>, --size <size>\t\tSize of the words in characters");
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
    println!("    -s <size>, --size <size>\t\tSize of the passwords in characters");
    println!("    -c <count>, --count <count>\t\tNumber of passwords to generate");
    println!("\n  The following options are optional:");
    println!("    -o <path>, --output <path>\t\tSave the passwords in a file");
    println!("    -O, --output-only <path>\t\tSave the passwords in a file and do not display it");
    println!("    -j, --json\t\t\t\tOutput in JSON format\n\t\t\t\t\tCombine with -o to save the json output in a file");
}
