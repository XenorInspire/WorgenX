// Internal crates.
use crate::{
    benchmark,
    error::{ArgError, SystemError, WorgenXError},
    json,
    password::{self, PasswordConfig},
    system,
    wordlist::{self, WordlistConfig, WordlistValues},
};

// External crates.
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use std::{
    env,
    fs::{File, OpenOptions},
    sync::{Arc, Mutex},
};

/// This struct is built from PasswordConfig and optional arguments that will be used to generate the random password.
///
struct PasswordGenerationOptions {
    password_config: PasswordConfig,
    json: bool,
    output_file: String,
    no_display: bool,
}

/// This struct is built from WordlistValues and optional arguments that will be used to generate the wordlist.
///
struct WordlistGenerationOptions {
    wordlist_values: WordlistValues,
    output_file: String,
    no_loading_bar: bool,
    threads: usize,
}

/// This struct is built from the arguments for the benchmark feature.
///
struct BenchmarkOptions {
    threads: usize,
}

/// This function is responsible for building the command context of the CLI mode with the clap framework.
///
/// # Returns
///
/// Command struct containing the different features of WorgenX.
///
fn build_command_context() -> Command {
    let default_threads: &'static str = Box::leak(num_cpus::get().to_string().into_boxed_str()); // Ensure a static reference to the number of logical cores of the CPU
    let wordlist_command: Command = Command::new("wordlist")
        .arg_required_else_help(true)
        .arg(
            Arg::new("lowercase_wordlist")
                .short('l')
                .long("lowercase")
                .help("Add lowercase characters to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("uppercase_wordlist")
                .short('u')
                .long("uppercase")
                .help("Add uppercase characters to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("numbers_wordlist")
                .short('n')
                .long("numbers")
                .help("Add numbers to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("special_characters_wordlist")
                .short('x')
                .long("special-characters")
                .help("Add special characters to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("mask")
                .short('m')
                .long("mask")
                .help("Mask used to generate the words")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("mask")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Save the wordlist in a text file")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("path")
                .required(true),
        )
        .arg(
            Arg::new("disable_loading_bar")
                .short('d')
                .long("disable-loading-bar")
                .help("Disable the loading bar when generating the wordlist")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("hash")
                .short('h')
                .long("hash")
                .help("Hash algorithm to use for the wordlist")
                .value_parser(vec!["md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3-224", "sha3-256", "sha3-384", "sha3-512", "blake2b", "blake2s", "whirlpool"])
                .value_name("hash"),
        )
        .arg(
            Arg::new("threads_wordlist")
                .short('t')
                .long("threads")
                .help("Number of threads to generate the passwords")
                .value_parser(value_parser!(usize))
                .value_name("threads")
                .default_value(default_threads),
        );

    let password_command: Command = Command::new("password")
        .arg_required_else_help(true)
        .arg(
            Arg::new("lowercase_password")
                .short('l')
                .long("lowercase")
                .help("Add lowercase characters to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("uppercase_password")
                .short('u')
                .long("uppercase")
                .help("Add uppercase characters to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("numbers_password")
                .short('n')
                .long("numbers")
                .help("Add numbers to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("special_characters_password")
                .short('x')
                .long("special-characters")
                .help("Add special characters to the words")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .help("Size of the passwords in characters")
                .value_parser(value_parser!(u32).range(1..i64::from(u32::MAX)))
                .value_name("size")
                .required(true),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .help("Number of passwords to generate")
                .value_parser(value_parser!(u64).range(1..u64::MAX))
                .value_name("count")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Save the passwords in a file")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("path")
                .conflicts_with("output_only"),
        )
        .arg(
            Arg::new("output_only")
                .short('O')
                .long("output-only")
                .help("Save the passwords only in a file, not in stdout")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_name("path")
                .conflicts_with("output"),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .help("Output in JSON format")
                .action(ArgAction::SetTrue),
        );

    let benchmark_command: Command = Command::new("benchmark")
        .arg(
            Arg::new("threads_benchmark")
                .short('t')
                .long("threads")
                .help("Number of threads to use for the CPU benchmark")
                .value_parser(value_parser!(usize))
                .value_name("threads")
                .default_value(default_threads),
        );

    Command::new("worgenX")
        .args_conflicts_with_subcommands(true)
        .allow_external_subcommands(true)
        .disable_help_flag(true) // Keep the help handling in the run() function
        .disable_version_flag(true) // Keep the version handling in the run() function
        .disable_help_subcommand(true) // Keep the help handling in the run() function
        .arg(Arg::new("version").short('v').long("version").action(ArgAction::SetTrue))
        .arg(Arg::new("help").short('h').long("help").action(ArgAction::SetTrue))
        .subcommand(wordlist_command)
        .subcommand(password_command)
        .subcommand(benchmark_command)
}

/// This function is responsible for scheduling the execution of the different features of the program according to the user's choices.
///
/// # Returns
///
/// Ok(()) if the program has been executed, WorgenXError otherwise.
///
pub fn run() -> Result<(), WorgenXError> {
    let mut command_context: Command = build_command_context();
    if let Ok(matches) = command_context.clone().try_get_matches() {
        // Call the `display_help()` function instead of clap help with the -h or --help arguments (better control of the help message).
        if matches.get_flag("help") {
            display_help();
            return Ok(());
        }
        // Call the `println!()` macro instead of clap version with the -v or --version arguments (better control of the version message).
        if matches.get_flag("version") {
            println!("WorgenX v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
    }

    command_context.build();
    match command_context.get_matches().subcommand() {
        Some(("wordlist", sub_matches)) => run_wordlist(sub_matches),
        Some(("password", sub_matches)) => run_passwd(sub_matches),
        Some(("benchmark", sub_matches)) => run_benchmark(sub_matches),
        _ => Err(WorgenXError::ArgError(ArgError::NoArgument))
    }
}

/// This function is responsible for scheduling the execution of the random password generation functions of the program.
///
/// # Arguments
///
/// * `sub_matches` - A reference to ArgMatches containing the arguments passed to the program.
///
/// # Returns
///
/// Ok(()) if the password has been generated, WorgenXError otherwise.
///
fn run_passwd(sub_matches: &ArgMatches) -> Result<(), WorgenXError> {
    let password_generation_parameters: PasswordGenerationOptions = allocate_passwd_config_cli(sub_matches)?;
    let passwords: Vec<String> = password::generate_random_passwords(
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
        println!("{all_passwords}");
    }

    if !password_generation_parameters.output_file.is_empty() {
        let file: File = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(password_generation_parameters.output_file.clone())
        {
            Ok(file) => file,
            Err(_) => {
                return Err(WorgenXError::SystemError(SystemError::UnableToCreateFile(
                    password_generation_parameters.output_file,
                    "Please check the path and try again".to_string(),
                )))
            }
        };
        let shared_file: Arc<Mutex<File>> = Arc::new(Mutex::new(file));
        system::save_passwd_to_file(&shared_file, &all_passwords)?;
    }

    Ok(())
}

/// This function is responsible for checking the values of the arguments passed to the program for the random password generation feature.
/// This function is called only if the user specifies the password command.
///
/// # Arguments
///
/// * `sub_matches` - A reference to ArgMatches containing the arguments passed to the program.
///
/// # Returns
///
/// PasswordGenerationOptions containing the password configuration and optional arguments, WorgenXError otherwise.
///
fn allocate_passwd_config_cli(sub_matches: &ArgMatches) -> Result<PasswordGenerationOptions, WorgenXError> {
    // Clap framework ensures that the arguments are present.
    let length: u32 = *sub_matches.get_one::<u32>("size").unwrap();
    let size: u64 = *sub_matches.get_one::<u64>("count").unwrap();
    
    let mut output_file: String = String::new();
    let mut json: bool = false;
    let mut no_display: bool = false;
    let mut password_config: PasswordConfig =
        PasswordConfig {
            numbers: false,
            special_characters: false,
            uppercase: false,
            lowercase: false,
            length,
            number_of_passwords: size,
        };

    update_config(&mut password_config.lowercase, sub_matches, "lowercase_password");
    update_config(&mut password_config.uppercase, sub_matches, "uppercase_password");
    update_config(&mut password_config.numbers, sub_matches, "numbers_password");
    update_config(&mut password_config.special_characters, sub_matches, "special_characters_password");
    update_config(&mut json, sub_matches, "json");
    update_config(&mut output_file, sub_matches, "output");
    update_config(&mut output_file, sub_matches, "output_only");

    if !output_file.is_empty(){
        output_file = check_output_arg(&output_file)?;
    }

    if sub_matches.get_one::<String>("output_only").is_some() {
        no_display = true;
    }

    if !password_config.lowercase
        && !password_config.uppercase
        && !password_config.numbers
        && !password_config.special_characters
    {
        return Err(WorgenXError::ArgError(ArgError::MissingConfiguration));
    }

    Ok(PasswordGenerationOptions {
        password_config,
        json,
        output_file,
        no_display,
    })
}

/// This function is responsible for scheduling the execution of the wordlist generation functions.
///
/// # Arguments
///
/// * `sub_matches` - A reference to ArgMatches containing the arguments passed to the program.
///
/// # Returns
///
/// Ok(()) if the wordlist has been generated, WorgenXError otherwise.
///
fn run_wordlist(sub_matches: &ArgMatches) -> Result<(), WorgenXError> {
    let wordlist_generation_parameters: WordlistGenerationOptions = allocate_wordlist_config_cli(sub_matches)?;

    let wordlist_config: WordlistConfig = wordlist::build_wordlist_config(&wordlist_generation_parameters.wordlist_values);
    let nb_of_passwords: u64 = wordlist_config.dict.len().pow(wordlist_config.mask_indexes.len() as u32) as u64;
    println!("Estimated size of the wordlist: {}", system::get_estimated_size(nb_of_passwords, wordlist_config.formated_mask.len() as u64));
    println!("Wordlist generation in progress...");

    wordlist::wordlist_generation_scheduler(
        &wordlist_config,
        nb_of_passwords,
        wordlist_generation_parameters.threads,
        &wordlist_generation_parameters.output_file,
        wordlist_generation_parameters.no_loading_bar,
    )
}

/// This function is responsible for checking the values of the arguments passed to the program.
/// This function is called only if the user specifies the wordlist command.
///
/// # Arguments
///
/// * `sub_matches` - A reference to ArgMatches containing the arguments passed to the program.
///
/// # Returns
///
/// WordlistGenerationOptions containing the wordlist configuration and optional arguments, WorgenXError otherwise.
///
fn allocate_wordlist_config_cli(sub_matches: &ArgMatches) -> Result<WordlistGenerationOptions, WorgenXError> {
    let mut output_file: String = String::new();
    let mut no_loading_bar: bool = false;
    let mut threads: usize = 0;
    let mut wordlist_values: WordlistValues = WordlistValues {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        mask: String::new(),
        hash: String::new(),
    };

    update_config(&mut wordlist_values.lowercase, sub_matches, "lowercase_wordlist");
    update_config(&mut wordlist_values.uppercase, sub_matches, "uppercase_wordlist");
    update_config(&mut wordlist_values.numbers, sub_matches, "numbers_wordlist");
    update_config(&mut wordlist_values.special_characters, sub_matches, "special_characters_wordlist");
    update_config(&mut wordlist_values.mask, sub_matches, "mask");
    update_config(&mut wordlist_values.hash, sub_matches, "hash");
    update_config(&mut output_file, sub_matches, "output");
    update_config(&mut no_loading_bar, sub_matches, "disable_loading_bar");
    update_config(&mut threads, sub_matches, "threads_wordlist");

    if !wordlist_values.lowercase
        && !wordlist_values.uppercase
        && !wordlist_values.numbers
        && !wordlist_values.special_characters
    {
        return Err(WorgenXError::ArgError(ArgError::MissingConfiguration));
    }

    Ok(WordlistGenerationOptions {
        wordlist_values,
        output_file,
        no_loading_bar,
        threads,
    })
}

/// This function is responsible for scheduling the execution of the benchmark functions of WorgenX.
/// It will display the number of passwords generated in 1 minute.
/// The benchmark is based on the generation of random passwords.
/// The profile used for the benchmark is defined in the benchmark module (`PASSWORD_CONFIG` constant).
///
/// # Arguments
///
/// * `sub_matches` - A reference to ArgMatches containing the arguments sent to the program.
///
/// # Returns
///
/// Ok(()) if the benchmark has correctly been executed, WorgenXError otherwise.
///
fn run_benchmark(sub_matches: &ArgMatches) -> Result<(), WorgenXError> {
    let benchmark_parameters: BenchmarkOptions = allocate_benchmark_config_cli(sub_matches);
    let result: u64 = benchmark::load_cpu_benchmark(benchmark_parameters.threads)?;
    
    println!("Your CPU has generated {result} passwords in 1 minute");
    Ok(())
}

/// This function is responsible for checking the syntax of the arguments sent to the program for the benchmark feature.
/// This function is called only if the user specifies the benchmark command.
///
/// # Arguments
///
/// * `sub_matches` - A reference to ArgMatches containing the arguments sent to the program.
///
/// # Returns
///
/// BenchmarkOptions struct containing the benchmark configuration.
///
fn allocate_benchmark_config_cli(sub_matches: &ArgMatches) -> BenchmarkOptions {
    let mut threads: usize = 0;
    update_config(&mut threads, sub_matches, "threads_benchmark");
    BenchmarkOptions { threads }
}

/// This function is responsible for checking the path for the 'output' arguments, if it's a valid path on the filesystem.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Returns
///
/// Ok(String) if the path is valid, WorgenXError otherwise.
///
fn check_output_arg(path: &str) -> Result<String, WorgenXError> {
    match system::is_valid_path(path) {
        Ok(full_path) => Ok(full_path),
        Err(e) => Err(WorgenXError::SystemError(e)),
    }
}

/// This function is responsible for updating the value of a field from a structure (ArgMatches from clap framwork) with the value of a CLI argument.
///
/// # Arguments
///
/// * `field` - The field to update.
/// * `sub_matches` - A reference to ArgMatches containing the arguments passed to the program.
/// * `key` - The key of the argument.
///
/// # Returns
///
/// The field updated with the value of the argument.
///
fn update_config<T: Clone + Send + Sync + 'static>(
    field: &mut T,
    sub_matches: &ArgMatches,
    key: &str,
) {
    if let Some(value) = sub_matches.get_one::<T>(key) {
        *field = value.clone();
    }
}

/// This function is responsible for displaying the help menu with all the features of WorgenX and their options.
///
fn display_help() {
    println!("Usage: worgenX <command> [options]");
    println!("Commands:");
    println!("  wordlist\t\tGenerate a wordlist");
    println!("  password\t\tGenerate random password(s)");
    println!("  benchmark\t\tCPU Benchmark");
    println!("  -v, --version\t\tDisplay the version of WorgenX");
    println!("  -h, --help\t\tDisplay this help message\n\n");
    println!("You can find below the options for the main features of WorgenX:\n");

    println!("  --- Wordlist generation ---");
    println!("  You must specify at least one of the following options: -l, -u, -n, -x");
    println!("    -l, --lowercase\t\t\tAdd lowercase characters to the words");
    println!("    -u, --uppercase\t\t\tAdd uppercase characters to the words");
    println!("    -n, --numbers\t\t\tAdd numbers to the words");
    println!("    -x, --special-characters\t\tAdd special characters to the words");
    println!("\n  These parameters are mandatory:");
    println!("    -m <mask>, --mask <mask>\t\tMask used to generate the words");
    println!("    -o <path>, --output <path>\t\tSave the wordlist in a text file");
    println!("\n  The following options are optional:");
    println!("    -d, --disable-loading-bar\t\tDisable the loading bar when generating the wordlist");
    println!("    -h, --hash <hash>\t\t\tHash algorithm to use for the wordlist.\n\t\t\t\t\tYou can choose between: md5, sha1, sha224, sha256, sha384, sha512,\n\t\t\t\t\tsha3-224, sha3-256, sha3-384, sha3-512, blake2b-512, blake2s-256 and whirlpool");
    println!("    -t <threads>, --threads <threads>\tNumber of threads to generate the passwords\n\t\t\t\t\tBy default, the number of threads is based on the number of logical cores of the CPU");

    println!("\n  --- Password generation ---");
    println!("  You must specify at least one of the following options: -l, -u, -n, -x");
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
    println!("    -t <threads>, --threads <threads>\tNumber of threads to use for the CPU benchmark\n\t\t\t\t\tBy default, the number of threads is based on the number of logical cores of the CPU\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_passwd_config_cli() {
        let command_context: Command = build_command_context();
        let matches: ArgMatches = command_context.get_matches_from(vec!["worgenX", "password", "-l", "-u", "-n", "-x", "-s", "10", "-c", "5", "-o", "test.txt", "-j"]);
        let (_, sub_matches) = matches.subcommand().unwrap();
        let result: PasswordGenerationOptions = allocate_passwd_config_cli(sub_matches).unwrap();
        
        assert_eq!(result.password_config.number_of_passwords, 5);
        assert_eq!(result.password_config.length, 10);
        assert!(result.password_config.lowercase);
        assert!(result.password_config.uppercase);
        assert!(result.password_config.numbers);
        assert!(result.password_config.special_characters);
        assert!(result.output_file.contains("test.txt"));
        assert!(result.json);
        assert!(!result.no_display);
    }

    #[test]
    fn test_allocate_wordlist_config_cli() {
        let command_context: Command = build_command_context();
        let matches: ArgMatches = command_context.get_matches_from(vec!["worgenX", "wordlist", "-l", "-u", "-n", "-x", "-m", "A?1", "-o", "test.txt", "-d", "-t", "4"]);
        let (_, sub_matches) = matches.subcommand().unwrap();
        let result: WordlistGenerationOptions = allocate_wordlist_config_cli(sub_matches).unwrap();
        
        assert_eq!(result.wordlist_values.mask, "A?1");
        assert_eq!(result.threads, 4_usize);
        assert!(result.wordlist_values.lowercase);
        assert!(result.wordlist_values.uppercase);
        assert!(result.wordlist_values.numbers);
        assert!(result.wordlist_values.special_characters);
        assert!(result.output_file.contains("test.txt"));
        assert!(result.no_loading_bar);       
    }

    #[test]
    fn test_allocate_benchmark_config_cli() {
        let command_context: Command = build_command_context();
        let matches: ArgMatches = command_context.get_matches_from(vec!["worgenX", "benchmark", "-t", "4"]);
        let (_, sub_matches) = matches.subcommand().unwrap();
        let result: BenchmarkOptions = allocate_benchmark_config_cli(sub_matches);
        
        assert_eq!(result.threads, 4_usize);
    }

    #[test]
    fn test_check_output_arg() {
        let result: Result<String, WorgenXError> = check_output_arg("./test.txt");
        assert!(result.is_ok());
    }
}