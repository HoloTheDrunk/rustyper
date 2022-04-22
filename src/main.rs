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

struct Statistics {
    correct: u32,
    incorrect: u32
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Invalid arguments.\nUsage: rustype path");
        return;
    }

    let text = fs::read_to_string(args[0].clone()).expect("Unable to read file");

    let (fitx, firx) = mpsc::channel::<Input>();
    let (fotx, forx) = mpsc::channel::<FrontMessage>();
    let frontend_thread = thread::spawn(move || frontend::run(fitx, forx));

    let (titx, tirx) = mpsc::channel::<TimerRequest>();
    let (totx, torx) = mpsc::channel::<TimerResponse>();
    let timer_thread = thread::spawn(move || timer::run(tirx, totx));

    main_loop(&firx, &fotx, &titx, &torx);

    if let Err(message) = fotx.send(FrontMessage::Exit) {
        eprintln!("Error terminating frontend: {}", message);
    }

    if frontend_thread.join().is_err() {
        eprintln!("Error joining frontend thread.");
    }
}

fn main_loop(
    firx: &Receiver<Input>,
    fotx: &Sender<FrontMessage>,
    titx: &Sender<TimerRequest>,
    torx: &Receiver<TimerResponse>,
) {
    if let Err(error) = titx.send(TimerRequest::Start) {
        eprintln!("Error starting timer: {}", error);
        return;
    }

    let mut stats = Statistics { correct: 0, incorrect: 0 };

    // Wait for Input from frontend
    while let Ok(received) = firx.recv() {
        // Send back the appropriate response after handling the Input
        if let Err(error) = fotx.send(match received {
            Input::KeyBackspace => FrontMessage::Backspace,
            //TODO change stats, check for validity against text, etc...
            Input::Character(_) => FrontMessage::Valid {
                character: received,
                wpm: 0.,
            },
            _ => continue,
        }) {
            eprintln!("{}", error);
            break;
        }
    }

    if let Err(error) = titx.send(TimerRequest::Stop) {
        eprintln!("Error starting timer: {}", error);
    }

    //TODO: Send final WPM
}

/*
fn inpt() {
  loop {
      match window.getch() {
          Some(Input::Character(c)) => { window.addch(c); },
          Some(Input::KeyDC) => break,
          Some(input) => { window.addstr(&format!("{:?}", input)); },
          None => ()
      }
  }
  endwin();
}
*/
