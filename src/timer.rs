use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

pub enum TimerRequest {
    Start,
    Get,
    Stop,
}

pub enum TimerResponse {
    Time {
        since_epoch: Duration,
        delta: Duration,
    },
    Stopped {
        time: Duration,
    },
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
                        TimerResponse::Time {
                            since_epoch: start.elapsed(),
                            delta: previous.elapsed(),
                        }
                    }
                    false => TimerResponse::Stopped {
                        time: previous - start,
                    },
                }) {
                    eprintln!("Error getting from timer: {}", error);
                    break;
                }
            }
            TimerRequest::Stop => running = false,
        }
    }
}
