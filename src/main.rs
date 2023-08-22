#[cfg(all(not(feature = "gui"), not(feature = "cli")))]
compile_error!("You must specify a mode: 'gui' or 'cli'.");

/// This constant is charged to store the program mode (GUI or CLI)
mod mode {
    #[cfg(feature = "cli")]
    pub mod cli;
    #[cfg(feature = "gui")]
    pub mod gui;
}

// Internal modules
mod dict;
mod error;
mod generator;
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
