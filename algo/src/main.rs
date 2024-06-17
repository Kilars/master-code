use crate::algorithm::{rest_main, Config, DpMode, Mode, RestMode};

pub mod algorithm;
pub mod dp;
pub mod dtw_band;
pub mod max_dtw;
pub mod rest;
pub mod spatial_filter;

fn run_config(conf: Config) -> Result<(), csv::Error> {
    rest_main(conf.clone(), false, 1)?;
    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let n = 100000;
    let dtw_dist = 200;
    let mut rest_mode = RestMode {
        rs: 100,
        compression_ratio: 3,
        spatial_filter: true,
        include_entire_trajectory: true,
        k: 3,
        error_point: 70,
    };
    let foo = vec![300, 400, 500, 600, 700, 800, 900, 1000];
    for f in &foo[1..] {
        rest_mode.rs = *f;
        run_config(Config {
            n,
            max_dtw_dist: dtw_dist,
            mode: Mode::Rest(rest_mode.clone()),
            dtw_band: 0,
        })?;
    }

    Ok(())
}
