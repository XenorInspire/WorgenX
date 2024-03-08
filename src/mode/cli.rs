// Internal crates
use crate::{
    benchmark,
    error::{ArgError, SystemError, WorgenXError},
    json,
    password::{self, PasswordConfig},
    system,
    wordlist::{self, WordlistValues},
};

// External crates
use std::{
    fs::OpenOptions,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Instant,
};

/// This struct is built from PasswordConfig and optional arguments will be used to generate the random password
///
struct PasswordGenerationOptions {
    password_config: PasswordConfig,
    json: bool,
    output_file: String,
    no_display: bool,
}

/// This struct is built from WordlistValues and optional arguments will be used to generate the wordlist
///
struct WordlistGenerationOptions {
    wordlist_values: WordlistValues,
    output_file: String,
    no_loading_bar: bool,
    threads: u64,
}

/// This struct is built from the arguments for the benchmark feature
///
struct BenchmarkOptions {
    threads: u64,
}

/// This function is charged to schedule in CLI mode the execution of the different features of the program
/// according to the user's choices
///
/// # Returns
///
/// Ok if the program has been executed, WorgenXError otherwise
///
pub fn run() -> Result<(), WorgenXError> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        return Err(WorgenXError::ArgError(ArgError::NoArgument));
    }

    match args[1].as_str() {
        "-w" | "--wordlist" => {
            if args.len() < 3 {
                return Err(WorgenXError::ArgError(ArgError::MissingConfiguration(
                    args[1].clone(),
                )));
            }
            match run_wordlist(&args) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        "-p" | "--passwd" => {
            if args.len() < 3 {
                return Err(WorgenXError::ArgError(ArgError::MissingConfiguration(
                    args[1].clone(),
                )));
            }
            match run_passwd(&args) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        "-b" | "--benchmark" => match run_benchmark(&args) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        },
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

/// This function is charged to schedule the execution of the random password generation feature of the program
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Returns
///
/// Ok if the password has been generated, WorgenXError otherwise
///
fn run_passwd(args: &[String]) -> Result<(), WorgenXError> {
    match allocate_passwd_config_cli(args) {
        Ok(password_generation_parameters) => {
            let passwords = password::generate_random_passwords(
                &password_generation_parameters.password_config,
            );
            let all_passwords: String = if password_generation_parameters.json {
                json::password_config_to_json(
                    &password_generation_parameters.password_config,
                    &passwords,
                )
            } else {
                passwords.join("\n")
            };

            if !password_generation_parameters.no_display {
                println!("{}", all_passwords);
            }

            if !password_generation_parameters.output_file.is_empty() {
                let file = match OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(password_generation_parameters.output_file.clone())
                {
                    Ok(file) => file,
                    Err(_) => {
                        return Err(WorgenXError::SystemError(SystemError::UnableToCreateFile(
                            password_generation_parameters.output_file.to_string(),
                            "Please check the path and try again".to_string(),
                        )))
                    }
                };
                let shared_file = Arc::new(Mutex::new(file));
                match system::save_passwd_to_file(shared_file, all_passwords) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Ok(())
        }

        Err(e) => Err(e),
    }
}

/// This function is charged to check the syntax of the arguments passed to the program for the random password generation feature
/// This function is called only if the user specifies the -p or --passwd argument
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Returns
///
/// PasswordGenerationOptions containing the password configuration and optional arguments or WorgenXError if an error occurs
///
fn allocate_passwd_config_cli(args: &[String]) -> Result<PasswordGenerationOptions, WorgenXError> {
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
                    match check_numerical_value(&args[i + 1], "-s or --size") {
                        Ok(value) => {
                            password_config.length = value;
                        }
                        Err(e) => {
                            return Err(e);
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
                    match check_numerical_value(&args[i + 1], "-c or --count") {
                        Ok(value) => {
                            password_config.number_of_passwords = value;
                        }
                        Err(e) => {
                            return Err(e);
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
                    match check_output_arg(&args[i + 1], "-o or --output") {
                        Ok(full_path) => {
                            output_file = full_path;
                        }
                        Err(e) => {
                            return Err(e);
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
                continue;
            }
            "-O" | "--output-only" => {
                if i + 1 < args.len() {
                    if one_path {
                        return Err(WorgenXError::ArgError(ArgError::BothOutputArguments));
                    }
                    match check_output_arg(&args[i + 1], "-O or --output-only") {
                        Ok(full_path) => {
                            output_file = full_path;
                        }
                        Err(e) => {
                            return Err(e);
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

    Ok(PasswordGenerationOptions {
        password_config,
        json,
        output_file,
        no_display,
    })
}

/// This function is charged to schedule the execution of the wordlist generation feature of the program
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Returns
///
/// Ok if the wordlist has been generated, WorgenXError otherwise
///
fn run_wordlist(args: &[String]) -> Result<(), WorgenXError> {
    match allocate_wordlist_config_cli(args) {
        Ok(wordlist_generation_parameters) => {
            let wordlist_config =
                wordlist::build_wordlist_config(&wordlist_generation_parameters.wordlist_values);
            // nb of passwd = pow(dict.len(), nb of '?')
            let nb_of_passwd = wordlist_config
                .dict
                .len()
                .pow(wordlist_config.mask_indexes.len() as u32)
                as u64;
            let (tx, rx) = mpsc::channel::<Result<u64, WorgenXError>>();
            let pb = Arc::new(Mutex::new(system::get_progress_bar()));
            let pb_clone = Arc::clone(&pb);
            let main_thread = thread::spawn(move || {
                let mut current_value = 0;
                println!("Wordlist generation in progress...");
                for received in rx {
                    match received {
                        Ok(value) => {
                            current_value += value;
                            if !wordlist_generation_parameters.no_loading_bar {
                                wordlist::build_wordlist_progress_bar(
                                    current_value,
                                    nb_of_passwd,
                                    &pb_clone,
                                )
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                    if current_value == nb_of_passwd {
                        break;
                    }
                }
                Ok(())
            });

            let start = Instant::now();
            match wordlist::wordlist_generation_scheduler(
                &wordlist_config,
                nb_of_passwd,
                wordlist_generation_parameters.threads,
                &wordlist_generation_parameters.output_file,
                &tx,
            ) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            };
            match main_thread.join() {
                Ok(_) => (),
                Err(e) => {
                    if let Some(err) = e.downcast_ref::<WorgenXError>() {
                        return Err(err.clone());
                    } else {
                        return Err(WorgenXError::SystemError(SystemError::ThreadError(
                            format!("{:?}", e),
                        )));
                    }
                }
            }
            println!(
                "\nWordlist generated in {}",
                system::get_elapsed_time(start)
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// This function is charged to check the syntax of the arguments passed to the program
/// It does not check the values of the arguments
/// This function is called only if the user specifies the -w or --wordlist argument
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Returns
///
/// WordlistGenerationOptions containing the wordlist configuration and optional arguments or WorgenXError if an error occurs
///
fn allocate_wordlist_config_cli(
    args: &[String],
) -> Result<WordlistGenerationOptions, WorgenXError> {
    let mut output_file = String::new();
    let mut no_loading_bar = false;
    let mut skip = false;
    let mut threads = num_cpus::get_physical() as u64;
    let mut wordlist_values = WordlistValues {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        mask: String::new(),
    };

    for i in 2..args.len() {
        if skip {
            skip = false;
            continue;
        }
        match args[i].as_str() {
            "-l" | "--lowercase" => {
                wordlist_values.lowercase = true;
            }
            "-u" | "--uppercase" => {
                wordlist_values.uppercase = true;
            }
            "-n" | "--numbers" => {
                wordlist_values.numbers = true;
            }
            "-x" | "--special-characters" => {
                wordlist_values.special_characters = true;
            }
            "-m" | "--mask" => {
                if i + 1 < args.len() {
                    if args[i + 1].starts_with('-') {
                        return Err(WorgenXError::ArgError(ArgError::MissingValue(
                            args[i].clone(),
                        )));
                    } else if !args[i + 1].contains('?') {
                        return Err(WorgenXError::ArgError(ArgError::InvalidMask()));
                    } else {
                        wordlist_values.mask = args[i + 1].clone();
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
                    match check_output_arg(&args[i + 1], "-o or --output") {
                        Ok(full_path) => {
                            output_file = full_path;
                        }
                        Err(e) => {
                            return Err(e);
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
            "-t" | "--threads" => {
                if i + 1 < args.len() {
                    match check_numerical_value(&args[i + 1], "-t or --threads") {
                        Ok(value) => {
                            threads = value;
                        }
                        Err(e) => {
                            return Err(e);
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
            "-d" | "--disable-loading-bar" => {
                no_loading_bar = true;
                continue;
            }
            _ => {
                return Err(WorgenXError::ArgError(ArgError::UnknownArgument(
                    args[i].clone(),
                )));
            }
        }
    }

    if !wordlist_values.lowercase
        && !wordlist_values.uppercase
        && !wordlist_values.numbers
        && !wordlist_values.special_characters
    {
        return Err(WorgenXError::ArgError(ArgError::MissingConfiguration(
            args[1].clone(),
        )));
    }

    if wordlist_values.mask.is_empty() {
        return Err(WorgenXError::ArgError(ArgError::MissingArgument(
            "-m or --mask".to_string(),
        )));
    }

    if output_file.is_empty() {
        return Err(WorgenXError::ArgError(ArgError::MissingArgument(
            "-o or --output".to_string(),
        )));
    }

    Ok(WordlistGenerationOptions {
        wordlist_values,
        output_file,
        no_loading_bar,
        threads,
    })
}

/// This function is charged to schedule the execution of the benchmark feature of the program
/// It will display the number of passwords generated in 1 minute
/// The benchmark is based on the generation of random passwords
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Returns
///
/// Ok if the benchmark has been executed, WorgenXError otherwise
///
fn run_benchmark(args: &[String]) -> Result<(), WorgenXError> {
    match allocate_benchmark_config_cli(args) {
        Ok(benchmark_parameters) => {
            match benchmark::load_cpu_benchmark(benchmark_parameters.threads) {
                Ok(results) => Ok(println!(
                    "Your CPU has generated {} passwords in 1 minute",
                    results
                )),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

/// This function is charged to check the syntax of the arguments passed to the program for the benchmark feature
/// This function is called only if the user specifies the -b or --benchmark argument
///
/// # Arguments
///
/// * `args` - A vector of String containing the arguments passed to the program
///
/// # Returns
///
/// BenchmarkOptions containing the benchmark configuration or WorgenXError if an error occurs
///
fn allocate_benchmark_config_cli(args: &[String]) -> Result<BenchmarkOptions, WorgenXError> {
    let mut threads = num_cpus::get_physical() as u64;
    let mut skip = false;

    for i in 2..args.len() {
        if skip {
            skip = false;
            continue;
        }
        match args[i].as_str() {
            "-t" | "--threads" => {
                if i + 1 < args.len() {
                    match check_numerical_value(&args[i + 1], "-t or --threads") {
                        Ok(value) => {
                            threads = value;
                        }
                        Err(e) => {
                            return Err(e);
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
            _ => {
                return Err(WorgenXError::ArgError(ArgError::UnknownArgument(
                    args[i].clone(),
                )));
            }
        }
    }

    Ok(BenchmarkOptions { threads })
}

/// This function is charged to check path for the 'output' arguments
///
/// # Arguments
///
/// * `path` - The path to check
/// * `arg` - The argument name
///
/// # Returns
///
/// Ok if the path is valid, WorgenXError otherwise
///
fn check_output_arg(path: &str, arg: &str) -> Result<String, WorgenXError> {
    if path.starts_with('-') {
        return Err(WorgenXError::ArgError(ArgError::MissingValue(
            arg.to_string(),
        )));
    }
    match system::is_valid_path(path.to_string()) {
        Ok(full_path) => Ok(full_path),
        Err(e) => Err(WorgenXError::SystemError(e)),
    }
}

/// This function is charged to check a numerical value for the 'size' and 'count' arguments
///
/// # Arguments
///
/// * `value` - The value to check
/// * `arg` - The argument name
///
/// # Returns
///
/// Ok if the value is valid, WorgenXError otherwise
///
fn check_numerical_value(value: &str, arg: &str) -> Result<u64, WorgenXError> {
    if value.starts_with('-') {
        return Err(WorgenXError::ArgError(ArgError::MissingValue(
            arg.to_string(),
        )));
    }
    match value.parse::<u64>() {
        Ok(value) => {
            if value == 0 {
                return Err(WorgenXError::ArgError(ArgError::InvalidNumericalValue(
                    value.to_string(),
                    arg.to_string(),
                )));
            }
            Ok(value)
        }
        Err(_) => Err(WorgenXError::ArgError(ArgError::InvalidNumericalValue(
            value.to_string(),
            arg.to_string(),
        ))),
    }
}

/// This function is charged to display the help menu with all the features of WorgenX
///
fn display_help() {
    println!("Usage: worgenX <command> [options]");
    println!("Commands:");
    println!("  -w, --wordlist\tGenerate a wordlist");
    println!("  -p, --passwd\t\tGenerate random password(s)");
    println!("  -b, --benchmark\tCPU Benchmark");
    println!("  -v, --version\t\tDisplay the version of WorgenX");
    println!("  -h, --help\t\tDisplay this help message\n\n");
    println!("You can find below the options for the main features of WorgenX:\n");

    println!("  --- Wordlist generation ---");
    println!("  You must specify at least one of the following options: -l, -u, -n, -s");
    println!("    -l, --lowercase\t\t\tAdd lowercase characters to the words");
    println!("    -u, --uppercase\t\t\tAdd uppercase characters to the words");
    println!("    -n, --numbers\t\t\tAdd numbers to the words");
    println!("    -x, --special-characters\t\tAdd special characters to the words");
    println!("\n  This parameter is mandatory:");
    println!("    -m <mask>, --mask <mask>\t\tMask used to generate the words");
    println!("    -o <path>, --output <path>\t\tSave the wordlist in a text file");
    println!("\n  The following options are optional:");
    println!("    -d, --disable-loading-bar\t\tDisable the loading bar when generating the wordlist");
    println!("    -t <threads>, --threads <threads>\tNumber of threads to use to generate the passwords\n\t\t\t\t\tBy default, the number of threads is based on the number of physical cores of the CPU");

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
    println!("    -O <path>, --output-only <path>\tSave the passwords only in a file, not in stdout");
    println!("    -j, --json\t\t\t\tOutput in JSON format\n\t\t\t\t\tCombine with -o to save the json output in a file");

    println!("\n  --- CPU Benchmark ---");
    println!("  The following option is optional:");
    println!("    -t <threads>, --threads <threads>\tNumber of threads to use for the CPU benchmark\n\t\t\t\t\tBy default, the number of threads is based on the number of physical cores of the CPU\n");
}
