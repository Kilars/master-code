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
                        if conf.dtw_band != 0 {
                            mode_name.push_str("-BND"); // Append "-BND"
                            mode_name.push_str(&conf.dtw_band.to_string());
                            // Convert dtw_band to String and append
                        }
                        mode_name
                    }
                    Mode::DP(_) => {
                        let mut mode_name = String::from("DP");
                        if conf.dtw_band != 0 {
                            mode_name.push_str("-BND"); // Append "-BND"
                            mode_name.push_str(&conf.dtw_band.to_string());
                            // Convert dtw_band to String and append
                        }
                        mode_name
                    }
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
    let n = 100000;
    let dtw_dist = 200;
    let mut rest_mode = RestMode {
        rs: 100,
        compression_ratio: 5,
        spatial_filter: false,
        include_entire_trajectory: false,
        k: 3,
        error_point: 70,
    };
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.include_entire_trajectory = true;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    Ok(())
}
