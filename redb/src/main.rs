use clap::Parser;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::sync::mpsc;
use anyhow::Result;

struct LazyInsert {
    index: u64,
    bin: Vec<u8>,
    space: String,
    notifier: oneshot::Sender<()>,
}
struct Reaper {
    db: Arc<redb::Database>,
    recv: mpsc::Receiver<LazyInsert>,
}
impl Reaper {
    fn table_def(space: &str) -> redb::TableDefinition<u64, Vec<u8>> {
        redb::TableDefinition::new(space)
    }
    fn reap(&self, du: Duration) -> Result<()> {
        // wait for a entry
        let fst = self.recv.recv()?;

        let tx = self.db.begin_write()?;
        let mut notifiers = vec![];

        // insert the first entry
        {
            let mut tbl = tx.open_table(Self::table_def(&fst.space))?;
            tbl.insert(fst.index, fst.bin)?;
            notifiers.push(fst.notifier);
        }

        while let Ok(e) = self.recv.try_recv() {
            let mut tbl = tx.open_table(Self::table_def(&e.space))?;
            tbl.insert(e.index, e.bin)?;
            notifiers.push(e.notifier);
        }
        tx.commit()?;

        for notifier in notifiers {
            notifier.send(()).unwrap();
        }
        Ok(())
    }
}

#[derive(Parser)]
struct Opts {
    n_lanes: usize,
    datalen: usize,
    #[clap(long, default_value = "10")]
    du: u16,
}
fn main() {
    let opts = Opts::parse();
    let (collector, q) = stats::Collector::new();

    std::fs::remove_file("db").ok();
    let db = Arc::new(redb::Database::create("db").unwrap());
    
    let (tx, rx) = mpsc::channel();
    let reaper = Reaper {
        db: db.clone(),
        recv: rx,
    };
    std::thread::spawn(move || {
        loop {
            reaper.reap(Duration::from_micros(100)).ok();
        }
    });

    for lane_id in 0..opts.n_lanes {
        let db = db.clone();
        let mut reporter = stats::Reporter::new(q.clone());
        let tx = tx.clone();

        std::thread::spawn(move || {
            let tblname = format!("id={lane_id}");
            let tbldef: redb::TableDefinition<u64, Vec<u8>> = redb::TableDefinition::new(&tblname);
            for i in 0.. {
                let k = i;
                let v = stats::randbytes(opts.datalen);

                reporter.start();

                if false {
                    let tx = db.begin_write().unwrap();
                    {
                        let mut tbl = tx.open_table(tbldef).unwrap();
                        tbl.insert(k, v).unwrap();
                    }
                    tx.commit().unwrap();
                } else {
                    let (tx1, rx1) = oneshot::channel();
                    let e = LazyInsert {
                        index: k,
                        bin: v,
                        space: tblname.clone(),
                        notifier: tx1,
                    };
                    tx.send(e).unwrap();
                    rx1.recv().unwrap();
                }

                reporter.stop(opts.datalen);
            }
        });
    }

    let du = Duration::from_secs(opts.du as u64);
    std::thread::sleep(du);
    eprintln!("{}", collector.show(du));
}
