use crate::{
    frontend::FrontMessage,
    timer::{TimerRequest, TimerResponse},
};
use pancurses::Input;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

/// Main loop of the program where all the test logic and wpm calculation
/// happen.
pub fn run(
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

/// Utility function for easy thread message passing checks with hidden error
/// printing.
///
/// # Example
/// ```
/// let (tx, rx) = mpsc::channel::<bool>();
///
/// // Suppose this thread handles signals received through the above channel
/// // in a loop.
/// let kìng = thread::spawn(move || /* ... */);
///
/// while send_message(tx, true) {
///     println!("Successfully sent a message to kìng.");
/// }
/// ```
fn send_message<T>(sender: &Sender<T>, message: T) -> bool {
    match sender.send(message) {
        Ok(_) => true,
        Err(error) => {
            eprintln!("{}", error);
            false
        }
    }
}
