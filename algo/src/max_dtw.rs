use crate::rest::Point;
use std::collections::HashMap;

pub fn max_dtw_band<'a>(
    ta: &'a [Point],
    tb: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
    band: usize,
) -> f64 {
    let y_range = (0..ta.len())
        .map(|x| {
            let min = (x as f32 - band as f32).max(0.0) as usize;
            let max = (x + band + 1).min(ta.len());
            (min, max)
        })
        .collect::<Vec<_>>();
    let x_marks_the_spot = if ta.len() < tb.len() {
        (ta.len(), (ta.len() + band).min(tb.len()))
    } else {
        ((tb.len() + band).min(ta.len()), tb.len())
    };

    let foo = max_dtw(
        &ta[..x_marks_the_spot.0],
        &tb[..x_marks_the_spot.1],
        map,
        Some(&y_range),
    );
    foo
}
pub fn max_dtw<'a>(
    ta: &'a [Point],
    tb: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
    y_range: Option<&Vec<(usize, usize)>>,
) -> f64 {
    match (ta, tb) {
        ([], []) => 0.0,
        ([.., a], [.., b]) => match y_range {
            Some(y_range) => {
                if y_range[ta.len() - 1].0 <= tb.len() - 1 && tb.len() - 1 < y_range[ta.len() - 1].1
                {
                    a.distance(b).max(q(ta, tb, map, Some(y_range)))
                } else {
                    f64::MAX
                }
            }
            None => a.distance(b).max(q(ta, tb, map, None)),
        },
        _ => f64::MAX,
    }
}
fn memo_or_calculate<'a>(
    st: &'a [Point],
    rt: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
    y_range: Option<&Vec<(usize, usize)>>,
) -> f64 {
    match map.get(&(st, rt)) {
        Some(&v) => v,
        None => {
            let result = max_dtw(st, rt, map, y_range);
            map.insert((&st, &rt), result);
            result
        }
    }
}
fn q<'a>(
    ta: &'a [Point],
    tb: &'a [Point],
    map: &mut HashMap<(&'a [Point], &'a [Point]), f64>,
    y_range: Option<&Vec<(usize, usize)>>,
) -> f64 {
    memo_or_calculate(except_last(ta), except_last(tb), map, y_range)
        .min(memo_or_calculate(except_last(ta), tb, map, y_range))
        .min(memo_or_calculate(ta, except_last(tb), map, y_range))
}

fn except_last(s: &[Point]) -> &[Point] {
    match s {
        [not_last @ .., _] => not_last,
        _ => &[],
    }
}
