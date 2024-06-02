use clap::Parser;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser)]
struct Opts {
    n_lanes: usize,
    datalen: usize,
    #[clap(long, default_value = "5")]
    du: u16,
}
fn main() {
    let opts = Opts::parse();
    let (collector, q) = stats::Collector::new();

    std::fs::remove_file("db").ok();
    let db = Arc::new(redb::Database::create("db").unwrap());
    
    let mut tx = db.begin_write().unwrap();
    tx.set_durability(redb::Durability::Eventual);
    tx.commit().unwrap();

    for lane_id in 0..opts.n_lanes {
        let db = db.clone();
        let mut reporter = stats::Reporter::new(q.clone());

        std::thread::spawn(move || {
            let tblname = format!("id={lane_id}");
            let tbldef: redb::TableDefinition<u64, Vec<u8>> = redb::TableDefinition::new(&tblname);
            for i in 0.. {
                let k = i;
                let v = stats::randbytes(opts.datalen);

                reporter.start();
                let tx = db.begin_write().unwrap();
                {
                    let mut tbl = tx.open_table(tbldef).unwrap();
                    tbl.insert(k, v).unwrap();
                }
                tx.commit().unwrap();
                reporter.stop(opts.datalen);
            }
        });
    }

    let du = Duration::from_secs(opts.du as u64);
    std::thread::sleep(du);
    eprintln!("{}", collector.show(du));
}
