// Prevents the use of unsafe code
#![forbid(unsafe_code)]

// Prevents the compilation of both modes which may cause conflicts
#[cfg(all(not(feature = "gui"), not(feature = "cli")))]
compile_error!("You must specify a mode: 'gui' or 'cli'.");

#[cfg(all(feature = "gui", feature = "cli"))]
compile_error!("You must specify only one mode: 'gui' or 'cli'.");

// Internal modules
mod benchmark;
mod dict;
mod error;
mod mode;
mod password;
mod system;
mod wordlist;

#[cfg(feature = "cli")]
mod json;

/// This function is the "entry point" of the program.
///
fn main() {
    #[cfg(feature = "gui")]
    mode::gui::run();

    #[cfg(feature = "cli")]
    if let Err(e) =  mode::cli::run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    std::process::exit(0);
}
