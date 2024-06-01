use clap::Parser;
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Parser)]
struct Opts {
    n_lanes: usize,
    datalen: usize,
    #[clap(long, default_value = "5")]
    du: u16,
}
fn main() {
    let opts = Opts::parse();
    let (collector, q) = Collector::new();

    std::fs::remove_dir_all("db").ok();
    std::fs::create_dir("db").unwrap();
    let db: sled::Db = sled::Config::new().path("db").open().unwrap();

    for lane_id in 0..opts.n_lanes {
        let tree = db.open_tree(format!("id={lane_id}")).unwrap();
        let mut reporter = Reporter::new(q.clone());
        std::thread::spawn(move || for i in 0.. {
            let k = i.to_string();
            let v = randbytes(opts.datalen);

            reporter.start();
            tree.insert(&k, v).unwrap();
            reporter.stop(opts.datalen);
        });
    }

    let du = Duration::from_secs(opts.du as u64);
    std::thread::sleep(du);
    eprintln!("{}", collector.show(du));
}

pub fn randbytes(n: usize) -> Vec<u8> {
    vec![0; n]
}

pub type Queue = Arc<SegQueue<Packet>>;

pub struct Packet {
    pub time: Duration,
    pub length: usize,
}

pub struct Collector {
    q: Queue,
}
impl Collector {
    pub fn new() -> (Self, Queue) {
        let q = Arc::new(SegQueue::new());
        let this = Self { q: q.clone() };
        (this, q)
    }

    pub fn show(self, du: Duration) -> String {
        let mut n = 0;
        let mut sum_time = Duration::ZERO;
        let mut sum_amt = 0; // bytes
        let n = self.q.len();
        let mut remaining = n;
        while remaining > 0 {
            let packet = self.q.pop().unwrap();
            sum_amt += packet.length;
            sum_time += packet.time;
            remaining -= 1;
        }

        let latency = sum_time / n as u32;
        // kB/s
        let throughput = sum_amt as f64 / du.as_millis() as f64;
        format!("throughput {throughput} kB/s, latency {latency:?}")
    }
}

pub struct Reporter {
    q: Queue,
    cur: Option<Instant>,
}
impl Reporter {
    pub fn new(q: Queue) -> Self {
        Self { q, cur: None }
    }

    pub fn start(&mut self) {
        assert!(self.cur.is_none());
        let now = Instant::now();
        self.cur = Some(now);
    }

    pub fn stop(&mut self, n: usize) {
        let old = self.cur.take().unwrap();
        let elapsed = old.elapsed();
        self.q.push(Packet {
            time: elapsed,
            length: n,
        })
    }
}
