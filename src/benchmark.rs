// Internal crates
use crate::{
    error::{SystemError, WorgenXError},
    password::{self, PasswordConfig},
    system,
};

// External crates
use indicatif::ProgressBar;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Instant,
};

/// This constant defines the length of the password for the CPU benchmark
///
const BENCHMARK_LENGTH_PASSWORD: u64 = 10000;

/// This function is charged to load the CPU benchmark
///
/// # Returns
///
/// * `Result<u64, WorgenXError>` - The number of passwords generated in 60 seconds, WorgenXError otherwise
///
pub fn load_cpu_benchmark() -> Result<u64, WorgenXError> {
    let (tx, rx) = mpsc::channel::<Result<u64, WorgenXError>>();
    let pb = Arc::new(Mutex::new(system::get_progress_bar()));
    let pb_clone = Arc::clone(&pb);

    let progress_bar_thread = thread::spawn(move || {
        println!("WorgenX CPU Benchmark is in progress...");
        for received in rx {
            match received {
                Ok(value) => {
                    build_wordlist_progress_bar(value, &pb_clone);
                    if value == 60 {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    });

    let password_config = PasswordConfig {
        numbers: true,
        special_characters: true,
        uppercase: true,
        lowercase: true,
        length: BENCHMARK_LENGTH_PASSWORD,
        number_of_passwords: 1,
    };

    let start = Instant::now();
    let mut nb_of_passwd: u64 = 0;
    while start.elapsed().as_secs() < 60 {
        password::generate_random_passwords(&password_config);
        nb_of_passwd += 1;
        tx.send(Ok(start.elapsed().as_secs())).unwrap_or(());
    }

    match tx.send(Ok(0)) {
        Ok(_) => match progress_bar_thread.join() {
            Ok(_) => {}
            Err(_) => {
                return Err(WorgenXError::SystemError(SystemError::ThreadError(
                    "Something went wrong with the progress bar thread".to_string(),
                )))
            }
        },
        Err(e) => {
            return Err(WorgenXError::SystemError(SystemError::ThreadError(
                e.to_string(),
            )))
        }
    }

    Ok(nb_of_passwd)
}

/// This function is charged to build the progress bar during the benchmark
///
/// # Arguments
///
/// * `seconds` - The number of seconds elapsed
/// * `pb` - The progress bar instance (from the indicatif crate)
///
fn build_wordlist_progress_bar(seconds: u64, pb: &Arc<Mutex<ProgressBar>>) {
    let mut pourcentage: u64 = (seconds * 100) / 60;
    if pourcentage == 0 {
        pourcentage += 1;
    }
    if let Ok(pb) = pb.try_lock() {
        if pourcentage < 100 {
            pb.set_position(pourcentage);
            pb.set_message("Loading...");
        } else {
            pb.set_position(100);
            pb.finish_with_message(String::from("Benchmark done!"));
        }
    }
}
