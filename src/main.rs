#[cfg(all(not(feature = "gui"), not(feature = "cli")))]
compile_error!("You must specify a mode: 'gui' or 'cli'.");

// Internal modules
mod dict;
mod error;
mod generator;
mod mode;
mod password;
mod system;

/// This function is the "entry point" of the program
///
fn main() {
    #[cfg(feature = "gui")]
    mode::gui::run();

    #[cfg(feature = "cli")]
    match mode::cli::run() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
