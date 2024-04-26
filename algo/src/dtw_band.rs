use crate::rest::Point;
use dtw_rs_band_fork::{Algorithm, DynamicTimeWarping, ParameterizedAlgorithm, Restriction};

pub fn dtw_band(ta: &[Point], tb: &[Point], band: usize) -> f64 {
    DynamicTimeWarping::with_closure_and_param(&ta, &tb, Point::distance, Restriction::Band(band))
        .distance()
}
pub fn dtw(ta: &[Point], tb: &[Point]) -> f64 {
    DynamicTimeWarping::with_closure(&ta, &tb, Point::distance).distance()
}
