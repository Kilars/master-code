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
    // Vanilla REST
    let mut rest_mode = RestMode {
        rs: 100,
        compression_ratio: 5,
        spatial_filter: false,
        include_entire_trajectory: true,
        k: 0,
        error_point: 0,
    };
    //REST
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    //REST_EXCL
    rest_mode.include_entire_trajectory = false;
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.include_entire_trajectory = true;
    rest_mode.spatial_filter = true;
    rest_mode.error_point = 35;
    //REST-SF35
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.error_point = 70;
    //REST-SF70
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.k = 3;
    //REST-SF70-KNN3
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    rest_mode.spatial_filter = false;
    rest_mode.error_point = 0;
    //REST-KNN3
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::Rest(rest_mode),
        dtw_band: 0,
    })?;
    //DP-DTW
    run_config(Config {
        n,
        max_dtw_dist: dtw_dist,
        mode: Mode::DP(DpMode {}),
        dtw_band: 0,
    })?;
    Ok(())
}
