use crate::rest::Point;
use std::collections::HashMap;

pub fn max_dtw<'a>(
    ta: &'a [Point],
    tb: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
) -> f64 {
    match (ta, tb) {
        ([], []) => 0.0,
        ([.., a], [.., b]) => a.distance(b).max(q(ta, tb, map)),
        _ => f64::MAX,
    }
}
fn memo_or_calculate<'a>(
    st: &'a [Point],
    rt: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
) -> f64 {
    match map.get(&(st, rt)) {
        Some(&v) => v,
        None => {
            let result = max_dtw(st, rt, map);
            map.insert((&st, &rt), result);
            result
        }
    }
}
fn q<'a>(
    ta: &'a [Point],
    tb: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
) -> f64 {
    memo_or_calculate(except_last(ta), except_last(tb), map)
        .min(memo_or_calculate(except_last(ta), tb, map))
        .min(memo_or_calculate(ta, except_last(tb), map))
}

fn except_last(s: &[Point]) -> &[Point] {
    match s {
        [not_last @ .., _] => not_last,
        _ => &[],
    }
}
