use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use rand::Rng;

#[derive(Debug)]
struct Event {
    process_id: u64,
    time: u64,
    value: u32,
}

struct Process {
    id: u64,
    time: AtomicU64,
}

impl Process {
    fn new(id: u64) -> Self {
        Self {
            id,
            time: AtomicU64::new(0),
        }
    }

    fn send_event(&self, value: u32, process: &Process) {
        let time = self.time.fetch_add(1, Ordering::SeqCst);
        let event = Event {
            process_id: self.id,
            value,
            time,
        };
        println!(
            "send event:: sender_process_id={} receiver_process_id={} value={} time={}",
            self.id, process.id, event.value, event.time
        );
        process.receive_event(event);
    }

    fn receive_event(&self, event: Event) {
        println!(
            "event received: receiver_process_id={} sender_process_id={} value={} time={}",
            self.id, event.process_id, event.value, event.time
        );
        self.time
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current_time| {
                let new_time = std::cmp::max(current_time, event.time) + 1;
                Some(new_time)
            })
            .expect("closure returned Some(), therefore fetch_update will never return Err()");
    }
}

fn main() {
    let p1 = Arc::new(Process::new(0));
    let p2 = Arc::new(Process::new(1));

    let thread_1 = {
        let p1_clone = Arc::clone(&p1);
        let p2_clone = Arc::clone(&p2);
        std::thread::spawn(move || {
            for i in 0..10 {
                let seconds = Duration::from_secs(rand::thread_rng().gen_range(0..=5));
                std::thread::sleep(seconds);
                p1_clone.send_event(i, &p2_clone);
            }
        })
    };

    let thread_2 = {
        let p1_clone = Arc::clone(&p1);
        let p2_clone = Arc::clone(&p2);
        std::thread::spawn(move || {
            for i in 0..10 {
                let seconds = Duration::from_secs(rand::thread_rng().gen_range(0..=5));
                std::thread::sleep(seconds);
                p2_clone.send_event(i, &p1_clone);
            }
        })
    };

    thread_1.join().unwrap();
    thread_2.join().unwrap();
}
