#[cfg(all(not(feature = "gui"), not(feature = "cli")))]
compile_error!("You must specify a mode: 'gui' or 'cli'.");

#[cfg(all(feature = "gui", feature = "cli"))]
compile_error!("You must specify only one mode: 'gui' or 'cli'.");

// Internal modules
mod dict;
mod error;
mod generator;
mod mode;
mod password;
mod system;

#[cfg(feature = "cli")]
mod json;

/// This function is the "entry point" of the program
///
fn main() {
    #[cfg(feature = "gui")]
    mode::gui::run();

    #[cfg(feature = "cli")]
    match mode::cli::run() {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
}
