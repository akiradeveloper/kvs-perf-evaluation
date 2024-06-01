use std::time::{Duration, Instant};
use std::sync::mpsc;
use clap::Parser;
use std::sync::Arc;
use crossbeam::queue::SegQueue;

#[derive(Parser)]
struct Opts {
    n: usize,
    du: u16,
}
fn main() {
    let opts = Opts::parse();
    let (stats, collector, tx) = Collector::new();
    collector.start();

    for lane_id in 0..opts.n {
        let mut reporter = Reporter::new(tx.clone());
        std::thread::spawn(move || {
            loop {
                reporter.start();
                std::thread::yield_now();
                reporter.stop();
            }
        });
    }

    std::thread::sleep(Duration::from_secs(opts.du as u64));
    eprintln!("{}", stats.show());
}

pub struct Stats {
    q: Arc<SegQueue<Duration>>,
}
impl Stats {
    pub fn show(self) -> String {
        format!("n={}", self.q.len())
    }
}

pub struct Packet(Duration);

struct Collector {
    q: Arc<SegQueue<Duration>>,
    rx: mpsc::Receiver<Packet>,
}
impl Collector {
    pub fn new() -> (Stats, Collector, mpsc::Sender<Packet>) {
        let (tx, rx) = mpsc::channel();
        let q = Arc::new(SegQueue::new());
        let stats = Stats { q: q.clone() };
        let collector = Collector {
            q,
            rx,
        };
        (stats, collector, tx)
    }

    pub fn start(self) {
        std::thread::spawn(move || {
            loop {
                let packet = self.rx.recv().unwrap();
                self.q.push(packet.0);
            }
        });
    }
}

pub struct Reporter {
    tx: mpsc::Sender<Packet>,
    cur: Option<Instant>,
}
impl Reporter {
    pub fn new(tx: mpsc::Sender<Packet>) -> Self {
        Self {
            tx,
            cur: None,
        }
    }

    pub fn start(&mut self) {
        assert!(self.cur.is_none());
        let now = Instant::now();
        self.cur = Some(now);
    }

    pub fn stop(&mut self) {
        let old = self.cur.take().unwrap();
        let elapsed = old.elapsed();
        self.tx.send(Packet(elapsed)).unwrap();
    }
}