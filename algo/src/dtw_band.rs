use crate::rest::Point;
use dtw_rs::{Algorithm, DynamicTimeWarping, ParameterizedAlgorithm, Restriction};

pub fn dtw_band(ta: &[Point], tb: &[Point]) -> f64 {
    DynamicTimeWarping::with_closure(ta, tb, Point::distance).distance()
    //    DynamicTimeWarping::with_closure_and_param(ta, tb, Point::distance, Restriction::Band(1))
    //        .distance()
}
