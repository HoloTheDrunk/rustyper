use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

pub enum TimerRequest {
    Start,
    Get,
    Stop,
    Exit,
}

pub enum TimerResponse {
    /// Time since epoch and delta since last query
    Time(Duration, Duration),
    /// Final time
    Stopped(Duration),
}

pub fn run(tirx: mpsc::Receiver<TimerRequest>, totx: mpsc::Sender<TimerResponse>) {
    let start = Instant::now();
    let mut previous = start;

    let mut running = false;

    while let Ok(request) = tirx.recv() {
        match request {
            TimerRequest::Start => running = true,
            TimerRequest::Get => {
                if let Err(error) = totx.send(match running {
                    true => {
                        previous = Instant::now();
                        TimerResponse::Time(start.elapsed(), previous.elapsed())
                    }
                    false => TimerResponse::Stopped(previous - start),
                }) {
                    eprintln!("Error getting from timer: {}", error);
                    break;
                }
            }
            TimerRequest::Stop => running = false,
            TimerRequest::Exit => break,
        }
    }
}
