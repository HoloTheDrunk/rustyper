use ascii_utils::Check;

use pancurses::{chtype, endwin, initscr, noecho, Input};

use std::sync::mpsc;

#[doc(hidden)]
const GREY_PAIR: chtype = 1;
const GREEN_PAIR: chtype = 2;
const RED_PAIR: chtype = 3;

/// Messages sent from other threads to the frontend thread.
pub enum FrontMessage {
    Valid { character: Input, wpm: f32 },
    Invalid { character: Input, wpm: f32 },
    Backspace,
    Return,
    Nothing,
    Exit,
}

/// Internal state of the frontend.
struct State {
    text: String,
    index: usize,
}

/// Runs the [pancurses] frontend until told to terminate by another thread.
///
/// `fitx` => **f**rontend **i**nput **t**ransmitter.
///
/// `forx` => **f**rontend **o**utput **r**eceiver.
///
/// # Example
/// ```
/// use frontend::FrontMessage;
///
/// let (fitx, _) = mpsc::channel<char>();
/// let (fotx, forx) = mpsc::channel<FrontMessage>();
/// let frontend_thread = thread::spawn(move || frontend::run(fitx, forx));
///
/// if let Err(message) = fotx.send(FrontMessage::Exit) {
///     eprintln!("{}", message);
/// }
///
/// if frontend_thread.join().is_err() {
///     eprintln!("Error joining frontend thread.");
/// }
/// ```
///
/// [pancurses]: https://docs.rs/pancurses/latest/pancurses
pub fn run(fitx: mpsc::Sender<Input>, forx: mpsc::Receiver<FrontMessage>, text: String) {
    let window = init_frontend();

    set_color(&window, GREY_PAIR, false, true);
    window.printw(text.clone());
    window.mv(0, 0);

    let mut state = State { text, index: 0 };

    loop {
        match window.getch() {
            Some(Input::Character(c)) => {
                if c.is_printable() {
                    if let Err(error) = fitx.send(Input::Character(c)) {
                        eprintln!("Error sending character {} to logic thread: {}", c, error);
                        break;
                    }
                }
            }
            Some(Input::KeyBackspace) => {
                if let Err(error) = fitx.send(Input::KeyBackspace) {
                    eprintln!("Error sending backspace to logic thread: {}", error);
                    break;
                }
            }
            Some(Input::KeyEnter) => {
                if let Err(error) = fitx.send(Input::KeyEnter) {
                    eprintln!("Error sending backspace to logic thread: {}", error);
                    break;
                }
            }
            _ => continue,
        }

        match forx.recv() {
            Ok(received) => match received {
                FrontMessage::Exit => break,
                _ => handle_message(&window, received, &mut state),
            },
            Err(_) => continue,
        }
    }

    endwin();
}

/// Initialize the [pancurses] [window].
///
/// [pancurses]: https://docs.rs/pancurses/latest/pancurses/
/// [window]: https://docs.rs/pancurses/latest/pancurses/struct.Window.html
fn init_frontend() -> pancurses::Window {
    let window = initscr();
    window.refresh();
    window.keypad(true);
    noecho();

    let mut bg = pancurses::COLOR_BLACK;

    pancurses::start_color();
    if pancurses::has_colors() {
        if pancurses::use_default_colors() == pancurses::OK {
            bg = -1;
        }

        pancurses::init_pair(GREY_PAIR as i16, pancurses::COLOR_WHITE, bg);
        pancurses::init_pair(GREEN_PAIR as i16, pancurses::COLOR_GREEN, bg);
        pancurses::init_pair(RED_PAIR as i16, pancurses::COLOR_RED, bg);
    }

    window
}

/// Handles a [FrontMessage] appropriately.
///
/// # Example
/// ```
/// let win = init_frontend();
/// let received = FrontMessage::Valid {
///     character: pancurses::Input::Character('c'),
///     wpm: 0.
/// };
///
/// // Should print a bold green 'c'.
/// handle_message(&win, received);
/// ```
fn handle_message(window: &pancurses::Window, received: FrontMessage, state: &mut State) {
    match received {
        FrontMessage::Valid { character, .. } => add_char(window, state, character, true),
        FrontMessage::Invalid { character, .. } => add_char(window, state, character, false),
        FrontMessage::Backspace => {
            state.index -= 1;

            // Move left one, re-add original character, move left again then
            // reset attributes for the reset character.
            let (y, x) = window.get_cur_yx();
            window.mvaddch(y, x - 1, state.text.chars().nth(state.index).unwrap());
            window.mv(y, x - 1);
            window.chgat(1, pancurses::A_NORMAL, GREY_PAIR as i16);
        }
        FrontMessage::Nothing => {}
        _ => eprintln!("Unhandled forx case"),
    };
}

/// Sets or unsets a color and optionally a boldness.
///
/// # Example
/// ```
/// let win = init_frontend();
///
/// set_color(&win, GREEN_PAIR, true, true);
///
/// win.addch('c'); // Should print a bold green 'c'
/// ```
fn set_color(window: &pancurses::Window, pair: chtype, bold: bool, enabled: bool) {
    if pancurses::has_colors() {
        let mut attr = pancurses::COLOR_PAIR(pair);

        if bold {
            attr |= pancurses::A_BOLD;
        }

        if enabled {
            window.attrset(attr);
        } else {
            window.attroff(attr);
        }
    }
}

/// Adds a char of a color corresponding to its validity.
///
/// # Example
/// ```
/// let win = init_frontend();
/// // Should print a bold green 'c'.
/// add_char(&win, Input::Character('c'), true);
/// ```
fn add_char(window: &pancurses::Window, state: &mut State, input: Input, valid: bool) {
    set_color(
        window,
        if valid { GREEN_PAIR } else { RED_PAIR },
        true,
        true,
    );

    if let Input::Character(c) = input {
        state.index += 1;

        if c == ' ' {
            let (y, x) = window.get_cur_yx();
            if !valid {
                window.chgat(1, pancurses::A_BOLD, RED_PAIR as i16);
            }
            window.mv(y, x + 1);
        } else {
            window.addch(c);
        }
    }
}
