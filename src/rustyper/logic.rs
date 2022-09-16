use crate::{
    frontend::FrontMessage,
    timer::{TimerRequest, TimerResponse},
};
use pancurses::Input;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

struct State<'text, 'channels> {
    text: &'text [char],
    validity: Vec<bool>,
    calculate_wpm: Box<dyn Fn(&Vec<bool>, &Duration) -> f32>,
    _firx: &'channels Receiver<Input>,
    fotx: &'channels Sender<FrontMessage>,
    titx: &'channels Sender<TimerRequest>,
    torx: &'channels Receiver<TimerResponse>,
}

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

    let validity: Vec<bool> = vec![];
    let calculate_wpm = |validity: &Vec<bool>, time: &Duration| {
        validity.iter().filter(|b: &&bool| **b).count() as f32 / 5. / (time.as_secs_f32() / 60.)
    };

    let mut state = State {
        text,
        validity,
        calculate_wpm: Box::new(calculate_wpm),
        _firx: firx,
        fotx,
        titx,
        torx,
    };

    // Wait for Input from frontend
    while let Ok(received) = firx.recv() {
        //FIXME: hangs when deleting more than there is to delete
        // Send back the appropriate response after handling the Input
        match received {
            Input::KeyBackspace => {
                if !state.validity.is_empty() {
                    state.validity.pop();
                    if !send_message(fotx, FrontMessage::Backspace) {
                        break;
                    }
                } else if !send_message(fotx, FrontMessage::Nothing) {
                    break;
                }

                continue;
            }
            Input::KeyEnter => {
                if handle_character('\n', received, &mut state).is_err() {
                    break;
                }
            }
            Input::Character(c) => {
                if handle_character(c, received, &mut state).is_err() {
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
            Ok(calculate_wpm(&state.validity, &time))
        } else {
            Err(())
        }
    } else {
        eprintln!("Error getting final time from timer.");
        Err(())
    }
}

fn handle_character(c: char, received: Input, state: &mut State) -> Result<(), ()> {
    let mut time = Duration::default();
    let mut deltas: Vec<Duration> = vec![];

    // Send time get request
    if let Err(error) = state.titx.send(TimerRequest::Get) {
        eprintln!("Error getting time from timer: {}", error);
        return Err(());
    }

    // Wait for response and handle appropriately
    if let Ok(state) = state.torx.recv() {
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
    state.validity.push(c == state.text[state.validity.len()]);

    // Set message to send
    let message = if *state.validity.last().unwrap() {
        FrontMessage::Valid {
            character: received,
            wpm: (state.calculate_wpm)(&state.validity, &time),
        }
    } else {
        FrontMessage::Invalid {
            character: received,
            wpm: (state.calculate_wpm)(&state.validity, &time),
        }
    };

    if !send_message(state.fotx, message) {
        eprintln!("Error sending message to frontend.");
        return Err(());
    }

    if state.validity.len() == state.text.len() {
        return Ok(());
    }

    Ok(())
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
