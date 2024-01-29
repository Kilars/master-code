use itertools::Itertools;
use serde::{de, Deserialize};
use std::fs::File;

#[derive(Deserialize)]
struct CsvTrajectory {
    id: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<[f32; 2]>,
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
}

// MAXDWT Dynamic programming
// Dnn is the total cost. T(p, q) is a subtrajectory of T from index p to q. We are comparing
// trajectories. Dnn is total but we are getting max
fn max_dtw(a: Option<&Vec<[f32; 2]>>, b: Option<&Vec<[f32; 2]>>) -> f32 {
    match (&a, &b) {
        (None, None) => 0.0,
        (None, _) => f32::INFINITY,
        (_, None) => f32::INFINITY,
        (Some(a_vec), Some(b_vec)) => {
            distance(a_vec.last(), b_vec.last()).max(q(a_vec.last(), b_vec.last()))
        }
    }
}
//Need other (Haversine) distance function for lat lng, but this will do as a placeholder
fn distance(a: Option<&[f32; 2]>, b: Option<&[f32; 2]>) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx.powi(2) + dy.powi(2)).sqrt()
}
fn q(a_vec: Option<&[f32; 2]>, b_vec: Option<&[f32; 2]>) -> f32 {
    max_dtw(a_vec, b_vec).max(max_dtw(a_vec, b_vec))
}
// MRT set - add all non redundant trajectories

fn main() -> Result<(), csv::Error> {
    let csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;

    Ok(())
}
