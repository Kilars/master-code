use itertools::Itertools;
use serde::{de, Deserialize};

#[derive(Deserialize)]
struct CsvTrajectory {
    id: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<(f32, f32)>,
}
#[derive(Clone)]
struct Point {
    lat: f32,
    lng: f32,
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
}

// MAXDWT Dynamic programming
// Dnn is the total cost. T(p, q) is a subtrajectory of T from index p to q. We are comparing
// trajectories. Dnn is total but we are getting max
fn max_dtw(a: &[Point], b: &[Point]) -> f32 {
    match (a, b) {
        ([], []) => 0.0,
        ([.., pa], [.., pb]) => pa.distance(pb).max(q(a, b)),
        _ => f32::INFINITY,
    }
}
//Need other (Haversine) distance function for lat lng, but this will do as a placeholder
impl Point {
    fn distance(&self, other: &Point) -> f32 {
        let dx = self.lat - other.lat;
        let dy = self.lng - other.lng;
        (dx.powi(2) + dy.powi(2)).sqrt()
    }
}

fn except_last(s: &[Point]) -> &[Point] {
    match s {
        [not_last @ .., _] => not_last,
        _ => &[],
    }
}

fn q(a: &[Point], b: &[Point]) -> f32 {
    max_dtw(except_last(a), except_last(b))
        .min(max_dtw(except_last(a), &b))
        .min(max_dtw(&a, except_last(b)))
}
// MRT set - add all non redundant trajectories

fn main() -> Result<(), csv::Error> {
    let csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;

    Ok(())
}
