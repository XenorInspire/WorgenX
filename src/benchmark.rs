// Internal crates.
use crate::{
    error::{SystemError, WorgenXError},
    password::{self, PasswordConfig},
    system,
};

// External crates.
use indicatif::ProgressBar;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// This constant structure is used to define the password configuration for the benchmark.
/// Update these values if you want to stress the CPU more or less.
///
const PASSWORD_CONFIG: PasswordConfig = PasswordConfig {
    numbers: true,
    special_characters: true,
    uppercase: true,
    lowercase: true,
    length: 10000,
    number_of_passwords: 1, // Don't change this value, it's used to generate a single password on each iteration.
};

/// This static variable is used to track the number of passwords generated.
/// It is used to update the progress bar.
/// It is wrapped in a AtomicU64 to avoid data sharing issues between the threads.
///
static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// This function is responsible for loading the CPU benchmark.
///
/// # Arguments
///
/// * `nb_of_threads` - The number of threads to use for the stress test.
///
/// # Returns
///
/// * `Result<u64, WorgenXError>` - The number of passwords generated in 60 seconds, WorgenXError otherwise.
///
pub fn load_cpu_benchmark(nb_of_threads: usize) -> Result<u64, WorgenXError> {
    let (tx_progress_bar, rx_progress_bar) = mpsc::channel::<Result<u64, WorgenXError>>();
    let pb: Arc<Mutex<ProgressBar>> = Arc::new(Mutex::new(system::get_progress_bar()));
    let pb_clone: Arc<Mutex<ProgressBar>> = Arc::clone(&pb);
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let shared_signal: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    
    let progress_bar_thread: JoinHandle<Result<(), WorgenXError>> = thread::spawn(move || {
        println!("WorgenX CPU Benchmark is in progress...");
        for received in rx_progress_bar {
            build_wordlist_progress_bar(received?, &pb_clone);
        }

        build_wordlist_progress_bar(60, &pb_clone);
        Ok(())
    });

    let start: Instant = Instant::now();
    for _ in 0..nb_of_threads {
        let shared_signal_rst: Arc<AtomicBool> = Arc::clone(&shared_signal);
        threads.push(thread::spawn(move || {
            run_stress_test(&shared_signal_rst).unwrap_or_else(|e| println!("{e}"));
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

    let _: Result<(), WorgenXError> = progress_bar_thread.join().map_err(|_| {
        WorgenXError::SystemError(SystemError::ThreadError(
            "Something went wrong with the progress bar thread".to_string(),
        ))
    })?;

    for thread in threads {
        thread.join().map_err(|_| {
            WorgenXError::SystemError(SystemError::ThreadError(
                "CPU Benchmark feature".to_string(),
            ))
        })?;
    }

    let nb_of_passwd: u64 = GLOBAL_COUNTER.load(Ordering::SeqCst);
    Ok(nb_of_passwd)
}

/// This function is responsible for running the stress test in a thread.
/// It generates a lot of passwords in a loop.
/// The number of passwords generated is stored in a shared counter.
///
/// # Arguments
///
/// * `stop_signal` - The stop signal to stop the stress test.
///
/// # Returns
///
/// Ok(()) if the stress test succeed, WorgenXError otherwise.
///
fn run_stress_test(stop_signal: &Arc<AtomicBool>) -> Result<(), WorgenXError> {
    let mut nb_of_passwd: u64 = 0;
    loop {
        if !stop_signal.load(Ordering::SeqCst) {
            GLOBAL_COUNTER.fetch_add(nb_of_passwd, Ordering::SeqCst);
            return Ok(());
        }

        password::generate_random_passwords(&PASSWORD_CONFIG);
        nb_of_passwd += 1;
    }
}

/// This function is responsible for building the progress bar during the benchmark.
///
/// # Arguments
///
/// * `seconds` - The number of seconds elapsed.
/// * `pb` - The progress bar instance (from the indicatif crate).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cpu_benchmark() {
        let result: Result<u64, WorgenXError> = load_cpu_benchmark(4);
        assert!(result.is_ok());
    }
}
