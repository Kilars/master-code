use crate::algorithm::{rest_main, Config, DpMode, Mode, RestMode};

pub mod algorithm;
pub mod dp;
pub mod dtw_band;
pub mod max_dtw;
pub mod rest;
pub mod spatial_filter;

fn run_config(conf: Config) -> Result<(), csv::Error> {
    rest_main(conf.clone(), false, 5)?;
    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let n = 100000;
    let dtw_dist = 200;
    let mut rest_mode = RestMode {
        rs: 100,
        compression_ratio: 5,
        spatial_filter: true,
        include_entire_trajectory: true,
        k: 0,
        error_point: 70,
    };
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.k = 3;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.k = 0;
    rest_mode.error_point = 120;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.k = 3;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;

    //XREST
    rest_mode.include_entire_trajectory = false;
    rest_mode.k = 0;
    rest_mode.error_point = 120;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.k = 3;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;

    rest_mode.k = 0;
    rest_mode.spatial_filter = false;
    rest_mode.error_point = 0;
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

    //DP DTW
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::DP(DpMode {}),
        dtw_band: 0,
    })?;

    Ok(())
}
