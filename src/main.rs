//! ASM TP10 -- Rustype: Creating a shell typing speed test.
//!
//! The authors of this code claim no approval from the [EPITA] School of
//! Engineering and Computer Science.
//! It is provided as-is for educational purposes by members of the ASM group.
//!
//! [EPITA]: https://epita.it

mod frontend;
use frontend::FrontMessage;

use std::{sync::mpsc, thread};

fn main() {
    println!("Hello, world!");

    let (fitx, firx) = mpsc::channel::<pancurses::Input>();
    let (fotx, forx) = mpsc::channel::<FrontMessage>();
    let frontend_thread = thread::spawn(move || frontend::run(fitx, forx));

    loop {
        if let Ok(received) = firx.try_recv() {
            if let Err(error) = fotx.send(FrontMessage::Valid {
                character: received,
                wpm: 0.,
            }) {
                eprintln!("{}", error);
                break;
            }
        }
    }

    if let Err(message) = fotx.send(FrontMessage::Exit) {
        eprintln!("{}", message);
    }

    if frontend_thread.join().is_err() {
        eprintln!("Error joining frontend thread.");
    }
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
