use itertools::Itertools;
use serde::{de, Deserialize};
use std::collections::HashSet;

#[derive(Deserialize)]
struct CsvTrajectory {
    id: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<(f32, f32)>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct Point {
    lat: f32,
    lng: f32,
}
#[derive(Clone, Eq, Hash, PartialEq)]
struct Trajectory {
    id: String,
    polyline: Vec<Point>,
}
//Need other (Haversine) distance function for lat lng, but this will do as a placeholder
impl Point {
    fn distance(&self, other: &Point) -> f32 {
        let dx = self.lat - other.lat;
        let dy = self.lng - other.lng;
        (dx.powi(2) + dy.powi(2)).sqrt()
    }
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
}

fn max_dtw(ta: &[Point], tb: &[Point]) -> f32 {
    match (ta, tb) {
        ([], []) => 0.0,
        ([.., a], [.., b]) => a.distance(b).max(q(ta, tb)),
        _ => f32::INFINITY,
    }
}

fn except_last(s: &[Point]) -> &[Point] {
    match s {
        [not_last @ .., _] => not_last,
        _ => &[],
    }
}

fn q(ta: &[Point], tb: &[Point]) -> f32 {
    max_dtw(except_last(&ta), except_last(tb))
        .min(max_dtw(except_last(ta), &tb))
        .min(max_dtw(&ta, except_last(tb)))
}
fn increment_until_failure(
    t: &[Point],
    rts: Vec<&[Point]>,
    err: f32,
    start: usize,
    end: usize,
) -> usize {
    //need to ensure the list is long enough
    for j in end..t.len() - end - 1 {
        for rt in &rts {
            if max_dtw(&t[start..j + 1], rt) > err {
                return j - 1;
            } else if j == t.len() - 1 {
                return j;
            }
        }
    }
    end
}
// MRT set - add all non redundant trajectories
//fn mrt_search(t: &[Point], rts: Vec<&[Point]>, err: f32) -> Vec<Vec<Point>> {
//    let mut local_mrt: Vec<Vec<Point>> = Vec::new();
//    assert!(t.len() >= 2);
//    for i in 0..t.len() - 2 {
//        for rt in &rts {
//            if max_dtw(&t[i..i + 1], rt) < err {
//                let end = increment_until_failure(t, rts.clone(), err, i, i + 1);
//                local_mrt.push(t[i..end].to_vec());
//                break;
//            }
//        }
//    }
//    local_mrt
//}

fn mrt_search(t: &[Point], rts: HashSet<&[Point]>, err: f32) -> Vec<Vec<Point>> {
    let mut m: HashSet<Trajectory> = HashSet::new();
    for i in 0..t.len() - 2 {
        let st = t[i..i + 1].to_vec();
        for rt in &rts {
            for i in 0..rt.len() - 2 {
                if max_dtw(&st, &rt[i..i + 1]) < err {}
            }
        }
        // m(t[i..i+1])<-MRT set for segment t[i..i+1]
    }
    todo!()
}

fn compression_ratio(t: &[Point], rts: HashSet<&[Point]>, spatial_deviation: f32) -> f32 {
    todo!()
}
fn create_rts(trajectories: Vec<Trajectory>, min_comp_ratio: f32) {
    let mut rts: HashSet<&[Point]> = HashSet::new();
    trajectories.iter().for_each(|t| {
        if compression_ratio(&t.polyline, rts, 100.0) >= min_comp_ratio {
            rts.insert(&t.polyline);
        }
    });
}

fn main() -> Result<(), csv::Error> {
    let csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;
    let mut reference_trajectories: Vec<Trajectory> = Vec::new();

    Ok(())
}
