use crate::algorithm::{rest_main, Config, DpMode, Mode, RestMode};
use std::io::Write;

pub mod algorithm;
pub mod dp;
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

    let res = rest_main(conf.clone(), false);

    match res {
        Ok(res) => {
            let _file_write_res = write!(
                file,
                "{},{},{},{},{}\n",
                conf.n,
                //conf.rs,
                //conf.compression_ratio,
                //conf.spatial_filter,
                match conf.mode {
                    Mode::Rest(rest_conf) => {
                        let mut mode_name = "REST".to_string();
                        if rest_conf.spatial_filter {
                            mode_name = "REST-SF".to_string() + &rest_conf.error_point.to_string();
                        }
                        mode_name
                    }
                    Mode::DP(_) => "DP".to_string(),
                },
                conf.max_dtw_dist,
                res.runtime.as_secs_f64(),
                res.avg_cr,
            );
        }
        Err(err) => println!("{:?}", err),
    };

    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let rest_mode = RestMode {
        rs: 1000,
        compression_ratio: 5,
        spatial_filter: true,
        dtw_band: 0,
        error_point: 5,
    };
    let _rest = Config {
        n: 100000,
        max_dtw_dist: 200,
        mode: Mode::Rest(rest_mode),
    };

    let dp_mode = DpMode {};
    let dp = Config {
        n: 1000,
        max_dtw_dist: 200,
        mode: Mode::DP(dp_mode),
    };
    run_config(dp)?;
    Ok(())
}
