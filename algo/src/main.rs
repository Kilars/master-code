use crate::rest::{mrt_search, TwoPrecisionFixedPointPoint};
use itertools::Itertools;
use serde::{de, Deserialize};
use std::collections::HashSet;
pub mod max_dtw;
pub mod rest;

#[derive(Deserialize)]
struct CsvTrajectory {
    id: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<(f32, f32)>,
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
}

fn main() -> Result<(), csv::Error> {
    let _csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;
    let first_poly = [
        TwoPrecisionFixedPointPoint {
            lat: -3212,
            lng: 3412,
        },
        TwoPrecisionFixedPointPoint {
            lat: -3213,
            lng: 3411,
        },
    ];
    mrt_search(&first_poly, HashSet::new(), 200);
    Ok(())
}
