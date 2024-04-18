use crate::algorithm::{rest_main, Config};
use std::io::Write;

pub mod algorithm;
pub mod max_dtw;
pub mod plot;
pub mod rest;
pub mod spatial_filter;

fn main() -> Result<(), csv::Error> {
    let conf = Config {
        n: 10000,
        rs: 10,
        compression_ratio: 5,
        spatial_filter: true,
        error_trajectories: 200,
        error_point: 20,
    };
    let mut file = std::fs::File::options()
        .create(true)
        .append(true)
        .open("out/output.txt")
        .expect("Failed to open or create the file");

    write!(file, "{:?}\n", conf).unwrap();

    let begin = std::time::Instant::now();

    let res = rest_main(conf);
    match res {
        Ok(res) => {
            println!(
                "Res {:.2?} avg_mdtw {:.4?} avg_cr {:.2?}",
                res.runtime, res.avg_mdtw, res.avg_cr
            )
        }
        Err(_) => {}
    }
    write!(file, "Time: {:.2?}\n", begin.elapsed()).unwrap();

    Ok(())
}
