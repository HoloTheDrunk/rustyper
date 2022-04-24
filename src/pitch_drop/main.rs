use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[doc(hidden)]
const LIFE_EXPECTANCY: u64 = 1_500;
#[doc(hidden)]
const LIQUID_QTY: u64 = 5;
#[doc(hidden)]
const PERIOD: u64 = 3_750;

#[doc(hidden)]
fn main() {
    let shared_state = Arc::new(Mutex::new(LIQUID_QTY));

    let experiment = {
        let cloned_state = Arc::clone(&shared_state);
        thread::spawn(move || pitch_drop(cloned_state, PERIOD))
    };

    let mut observer: usize = 0;
    loop {
        if let Ok(state) = shared_state.lock() {
            if *state == 0 {
                println!("Experiment ended in observer {}'s lifetime.", observer);
                break;
            } else {
                println!("Remaining experiment duration: {}.", *state);
            }
        }

        observer += 1;
        println!("Giving the experiment to observer {}.", observer);

        thread::sleep(Duration::from_millis(LIFE_EXPECTANCY));
    }

    if experiment.join().is_err() {
        eprintln!("Error joining experiment thread");
    }
}

/// Reduces the state value by 1 every period.
fn pitch_drop(state: Arc<Mutex<u64>>, period: u64) {
    loop {
        if let Ok(mut state) = state.lock() {
            *state -= 1;
            println!("\x1b[34mplop\x1b[0m");

            if *state == 0 {
                break;
            }
        }

        thread::sleep(Duration::from_millis(period));
    }
}
