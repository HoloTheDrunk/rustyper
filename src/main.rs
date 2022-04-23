//! ASM TP10 -- Rustype: Creating a shell typing speed test.
//!
//! The authors of this code claim no approval from the [EPITA] School of
//! Engineering and Computer Science.
//! It is provided as-is for educational purposes by members of the ASM group.
//!
//! [EPITA]: https://epita.it

mod frontend;
use frontend::FrontMessage;

mod timer;
use timer::{TimerRequest, TimerResponse};

use pancurses::Input;

use std::{
    fs,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
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

    let result = main_loop(
        &firx,
        &fotx,
        &titx,
        &torx,
        &text.chars().map(|c| c as char).collect::<Vec<char>>(),
    );

    terminate_thread(frontend_thread, fotx, FrontMessage::Exit);
    terminate_thread(timer_thread, titx, TimerRequest::Exit);

    println!("Text: {}", text);

    if let Ok(wpm) = result {
        println!(" WPM: {:.2}", wpm);
    } else {
        eprintln!("Main loop returned an error.");
    }
}

fn main_loop(
    firx: &Receiver<Input>,
    fotx: &Sender<FrontMessage>,
    titx: &Sender<TimerRequest>,
    torx: &Receiver<TimerResponse>,
    text: &[char],
) -> Result<f32, ()> {
    if let Err(error) = titx.send(TimerRequest::Start) {
        eprintln!("Error starting timer: {}", error);
        return Err(());
    }

    let mut validity: Vec<bool> = vec![];
    let calculate_wpm = |validity: &Vec<bool>, time: &Duration| {
        validity.iter().filter(|b: &&bool| **b).count() as f32 / 5. / (time.as_secs_f32() / 60.)
    };

    // Wait for Input from frontend
    while let Ok(received) = firx.recv() {
        //FIXME: hangs when deleting more than there is to delete
        // Send back the appropriate response after handling the Input
        match received {
            Input::KeyBackspace => {
                if !validity.is_empty() {
                    validity.pop();
                    if !send_message(fotx, FrontMessage::Backspace) {
                        break;
                    }
                } else if !send_message(fotx, FrontMessage::Nothing) {
                    break;
                }

                continue;
            }
            Input::Character(c) => {
                let mut time = Duration::default();
                let mut deltas: Vec<Duration> = vec![];

                // Send time get request
                if let Err(error) = titx.send(TimerRequest::Get) {
                    eprintln!("Error getting time from timer: {}", error);
                    break;
                }

                // Wait for response and handle appropriately
                if let Ok(state) = torx.recv() {
                    match state {
                        TimerResponse::Time(current, delta) => {
                            time = current;
                            deltas.push(delta);
                        }
                        TimerResponse::Stopped(last) => {
                            eprintln!("Not running, last recorded time is {}s", last.as_secs())
                        }
                    }
                }

                // Check if the typed character is correct
                validity.push(c == text[validity.len()]);

                // Set message to send
                let message = if *validity.last().unwrap() {
                    FrontMessage::Valid {
                        character: received,
                        wpm: calculate_wpm(&validity, &time),
                    }
                } else {
                    FrontMessage::Invalid {
                        character: received,
                        wpm: calculate_wpm(&validity, &time),
                    }
                };

                if !send_message(fotx, message) {
                    eprintln!("Error sending message to frontend.");
                    break;
                }

                if validity.len() == text.len() {
                    break;
                }
            }
            _ => continue,
        }
    }

    if let Err(error) = titx.send(TimerRequest::Stop) {
        eprintln!("Error stopping timer: {}", error);
    }

    if titx.send(TimerRequest::Get).is_ok() {
        if let Ok(TimerResponse::Stopped(time)) = torx.recv() {
            Ok(calculate_wpm(&validity, &time))
        } else {
            Err(())
        }
    } else {
        eprintln!("Error getting final time from timer.");
        Err(())
    }
}

fn send_message<T>(sender: &Sender<T>, message: T) -> bool {
    match sender.send(message) {
        Ok(_) => true,
        Err(error) => {
            eprintln!("{}", error);
            false
        }
    }
}

fn terminate_thread<T>(thread: thread::JoinHandle<()>, sender: Sender<T>, kill_signal: T) {
    if let Err(message) = sender.send(kill_signal) {
        eprintln!("Error terminating frontend: {}", message);
    }

    if thread.join().is_err() {
        eprintln!("Error joining frontend thread.");
    }
}
