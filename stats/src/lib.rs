use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn randbytes(n: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut out = vec![0; n];
    rng.fill(&mut out[..]);
    out
}

pub type Queue = Arc<SegQueue<Packet>>;

pub struct Packet {
    time: Duration,
    length: usize,
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
        // MB/s
        let throughput = sum_amt as f64 / du.as_micros() as f64;
        format!("throughput={throughput} MB/s, latency={latency:?}")
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
