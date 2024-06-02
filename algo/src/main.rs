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
                match conf.mode {
                    Mode::Rest(rest_conf) => {
                        let mut mode_name = String::from("REST"); // Change to mutable String
                        if rest_conf.spatial_filter {
                            mode_name.push_str("-SF"); // Use push_str to append
                            mode_name.push_str(&rest_conf.error_point.to_string());
                            // Convert error_point to String and append
                        }
                        if rest_conf.dtw_band != 0 {
                            mode_name.push_str("-BND"); // Append "-BND"
                            mode_name.push_str(&rest_conf.dtw_band.to_string());
                            // Convert dtw_band to String and append
                        }
                        mode_name
                    }
                    Mode::DP(_) => String::from("DP"),
                },
                conf.n,
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
    let n = 10000;
    let dtw_dist = 200;
    let mut rest = RestMode {
        rs: 100,
        compression_ratio: 5,
        spatial_filter: true,
        error_point: 70,
        dtw_band: 20,
        k: 1,
    };
    run_config(Config {
        n,
        mode: Mode::Rest(rest),
        max_dtw_dist: dtw_dist,
    })?;
    rest.k = 2;
    run_config(Config {
        n,
        mode: Mode::Rest(rest),
        max_dtw_dist: dtw_dist,
    })?;
    rest.k = 3;
    run_config(Config {
        n,
        mode: Mode::Rest(rest),
        max_dtw_dist: dtw_dist,
    })?;
    rest.k = 5;
    run_config(Config {
        n,
        mode: Mode::Rest(rest),
        max_dtw_dist: dtw_dist,
    })?;
    rest.k = 7;
    run_config(Config {
        n,
        mode: Mode::Rest(rest),
        max_dtw_dist: dtw_dist,
    })?;
    Ok(())
}
