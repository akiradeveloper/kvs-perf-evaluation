use clap::Parser;
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

    std::fs::remove_dir_all("db").ok();
    std::fs::create_dir("db").unwrap();
    let db: sled::Db = sled::Config::new()
        .path("db")
        .flush_every_ms(Some(100))
        .open()
        .unwrap();

    for lane_id in 0..opts.n_lanes {
        let tree = db.open_tree(format!("id={lane_id}")).unwrap();
        let mut reporter = stats::Reporter::new(q.clone());
        std::thread::spawn(move || {
            for i in 0.. {
                let k = i.to_string();
                let v = stats::randbytes(opts.datalen);

                reporter.start();
                tree.insert(&k, v).unwrap();
                reporter.stop(opts.datalen);
            }
        });
    }

    let du = Duration::from_secs(opts.du as u64);
    std::thread::sleep(du);
    eprintln!("{}", collector.show(du));
}
