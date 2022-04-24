//! ASM TP10 -- Rustype: Creating a shell typing speed test.
//!
//! The authors of this code claim no approval from the [EPITA] School of
//! Engineering and Computer Science.
//! It is provided as-is for educational purposes by members of the ASM group.
//!
//! [EPITA]: https://epita.it

mod frontend;
use frontend::FrontMessage;

mod logic;

mod timer;
use timer::{TimerRequest, TimerResponse};

use pancurses::Input;

use std::{
    fs,
    sync::mpsc::{self, Sender},
    thread,
};

#[doc(hidden)]
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.is_empty() {
        eprintln!("Invalid arguments.\nUsage: rustype path");
        return;
    }

    let text = fs::read_to_string(&args[1])
        .expect("Unable to read file")
        .trim()
        .to_string();

    let text_copy = text.clone();
    let (fitx, firx) = mpsc::channel::<Input>();
    let (fotx, forx) = mpsc::channel::<FrontMessage>();
    let frontend_thread = thread::spawn(move || frontend::run(fitx, forx, text_copy));

    let (titx, tirx) = mpsc::channel::<TimerRequest>();
    let (totx, torx) = mpsc::channel::<TimerResponse>();
    let timer_thread = thread::spawn(move || timer::run(tirx, totx));

    let result = logic::run(
        &firx,
        &fotx,
        &titx,
        &torx,
        &text.chars().map(|c| c as char).collect::<Vec<char>>(),
    );

    terminate_thread(frontend_thread, &fotx, FrontMessage::Exit);
    terminate_thread(timer_thread, &titx, TimerRequest::Exit);

    println!("Text: {}", text);

    if let Ok(wpm) = result {
        println!(" WPM: {:.2}", wpm);
    } else {
        eprintln!("Main loop returned an error.");
    }
}

/// Utility function for easy thread termination and error handling.
///
/// # Example
/// ```
/// let (tx, rx) = mpsc::channel::<bool>();
///
/// // Suppose this thread handles signals received through the above channel
/// // in a loop.
/// let kìng = thread::spawn(move || /* ... */ );
///
/// terminate_thread(kìng, tx, false);
/// ```
fn terminate_thread<T>(thread: thread::JoinHandle<()>, sender: &Sender<T>, kill_signal: T) {
    if let Err(message) = sender.send(kill_signal) {
        eprintln!("Error terminating frontend: {}", message);
    }

    if thread.join().is_err() {
        eprintln!("Error joining frontend thread.");
    }
}
