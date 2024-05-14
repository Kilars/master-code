use crate::algorithm::{rest_main, Config};
use std::io::Write;

pub mod algorithm;
pub mod dtw_band;
pub mod max_dtw;
pub mod rest;
pub mod spatial_filter;

fn run_config(conf: Config) -> Result<(), csv::Error> {
    let mut file = std::fs::File::options()
        .create(true)
        .append(true)
        .open("out/output.txt")
        .expect("Failed to open or create the file");

    let res = rest_main(conf.clone(), true);

    match res {
        Ok(res) => {
            let _file_write_res = write!(
                file,
                "{},{},{},{},{},{},{},{}\n",
                conf.n,
                conf.rs,
                conf.compression_ratio,
                conf.spatial_filter,
                conf.error_trajectories,
                conf.error_point,
                res.runtime.as_secs_f64(),
                res.avg_cr,
            );
        }
        Err(err) => println!("{:?}", err),
    };

    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let config_base = Config {
        n: 100000,
        rs: 1000,
        compression_ratio: 5,
        spatial_filter: true,
        dtw_band: 0,
        error_trajectories: 200,
        error_point: 5,
    };
    let rs_seq = vec![15, 20, 25, 30, 10, 5];

    for rs in rs_seq {
        let mut conf = config_base.clone();
        conf.error_point = rs;
        run_config(conf.clone())?;
        //conf.spatial_filter = false;
        //run_config(conf)?;
    }
    Ok(())
}
