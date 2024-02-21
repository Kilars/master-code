use crate::rest::Point;

pub fn max_dtw(ta: &[Point], tb: &[Point]) -> f64 {
    match (ta, tb) {
        ([], []) => 0.0,
        ([.., a], [.., b]) => a.distance(b).max(q(ta, tb)),
        _ => f64::MAX,
    }
}

fn q(ta: &[Point], tb: &[Point]) -> f64 {
    max_dtw(except_last(&ta), except_last(tb))
        .min(max_dtw(except_last(ta), &tb))
        .min(max_dtw(&ta, except_last(tb)))
}

fn except_last(s: &[Point]) -> &[Point] {
    match s {
        [not_last @ .., _] => not_last,
        _ => &[],
    }
}
