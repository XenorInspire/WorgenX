// Internal crates
use crate::{
    error::{SystemError, WorgenXError},
    password::{self, PasswordConfig},
    system,
};

// External crates
use indicatif::ProgressBar;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// This constant structure is used to define the password configuration for the benchmark
/// Update these values if you want to stress the CPU more or less
///
const PASSWORD_CONFIG: password::PasswordConfig = PasswordConfig {
    numbers: true,
    special_characters: true,
    uppercase: true,
    lowercase: true,
    length: 10000,
    number_of_passwords: 1, // Don't change this value, it's used to generate a single password on each iteration
};

/// This function is charged to load the CPU benchmark
///
/// # Arguments
///
/// * `nb_of_threads` - The number of threads to use for the stress test
///
/// # Returns
///
/// * `Result<u64, WorgenXError>` - The number of passwords generated in 60 seconds, WorgenXError otherwise
///
pub fn load_cpu_benchmark(nb_of_threads: u8) -> Result<u64, WorgenXError> {
    let (tx_progress_bar, rx_progress_bar) = mpsc::channel::<Result<u64, WorgenXError>>();
    let pb: Arc<Mutex<ProgressBar>> = Arc::new(Mutex::new(system::get_progress_bar()));
    let pb_clone: Arc<Mutex<ProgressBar>> = Arc::clone(&pb);
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let shared_signal: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let shared_passwd_counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    let progress_bar_thread = thread::spawn(move || {
        println!("WorgenX CPU Benchmark is in progress...");
        for received in rx_progress_bar {
            match received {
                Ok(value) => {
                    build_wordlist_progress_bar(value, &pb_clone);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        build_wordlist_progress_bar(60, &pb_clone);
        Ok(())
    });

    let start = Instant::now();
    for _ in 0..nb_of_threads {
        let shared_signal_rst = Arc::clone(&shared_signal);
        let shared_passwd_counter: Arc<Mutex<u64>> = Arc::clone(&shared_passwd_counter);
        threads.push(thread::spawn(move || {
            let shared_passwd_counter = Arc::clone(&shared_passwd_counter);
            match run_stress_test(shared_signal_rst, shared_passwd_counter) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
        }));
    }

    while start.elapsed().as_secs() <= 60 {
        tx_progress_bar
            .send(Ok(start.elapsed().as_secs()))
            .unwrap_or(());
        thread::sleep(Duration::from_millis(500));
    }
    shared_signal.store(false, Ordering::SeqCst); // Stop the stress test
    drop(tx_progress_bar); // Drop the channel to stop the progress bar thread

    match progress_bar_thread.join() {
        Ok(_) => {}
        Err(_) => {
            return Err(WorgenXError::SystemError(SystemError::ThreadError(
                "Something went wrong with the progress bar thread".to_string(),
            )))
        }
    }

    for thread in threads {
        match thread.join() {
            Ok(_) => {}
            Err(_) => {
                return Err(WorgenXError::SystemError(SystemError::ThreadError(
                    "CPU Benchmark feature".to_string(),
                )))
            }
        }
    }

    let nb_of_passwd = match shared_passwd_counter.lock() {
        Ok(counter) => *counter,
        Err(_) => {
            return Err(WorgenXError::SystemError(SystemError::ThreadError(
                "Something went wrong with the shared counter".to_string(),
            )))
        }
    };
    Ok(nb_of_passwd)
}

/// This function is charged to run the stress test in a thread
/// It generates a lot of passwords in a loop
/// The number of passwords generated is stored in a shared counter
///
/// # Arguments
///
/// * `stop_signal` - The stop signal to stop the stress test
/// * `shared_passwd_counter` - The shared counter to store the number of passwords generated
///
/// # Returns
///
/// Ok(()) if the stress test is done, WorgenXError otherwise
///
fn run_stress_test(
    stop_signal: Arc<AtomicBool>,
    shared_passwd_counter: Arc<Mutex<u64>>,
) -> Result<(), WorgenXError> {
    let mut nb_of_passwd: u64 = 0;
    loop {
        if !stop_signal.load(Ordering::SeqCst) {
            match shared_passwd_counter.lock() {
                Ok(mut counter) => {
                    *counter += nb_of_passwd;
                    return Ok(());
                }
                Err(_) => {
                    return Err(WorgenXError::SystemError(SystemError::ThreadError(
                        "Something went wrong with the a stress test thread".to_string(),
                    )))
                }
            }
        }

        password::generate_random_passwords(&PASSWORD_CONFIG);
        nb_of_passwd += 1;
    }
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
