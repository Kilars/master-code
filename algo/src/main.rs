use crate::rest::ReferenceSet;
use itertools::Itertools;
use serde::{de, Deserialize};
use std::collections::HashSet;
pub mod max_dtw;
pub mod rest;

#[derive(Deserialize)]
struct CsvTrajectory {
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<(f32, f32)>,
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
}

fn main() -> Result<(), csv::Error> {
    let csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;

    let mut mrt_set = ReferenceSet(HashSet::new());

    // Generate reference set
    for traj in csv_trajectories {
        let t = traj
            .polyline
            .iter()
            .map(|&pnt| pnt.into())
            .collect::<Vec<_>>();
        let (_encoded_t, compression_ratio) = mrt_set.encode(&t, 0.2);
        if compression_ratio < 5.0 {
            mrt_set.0.insert(t);
        }
    }
    Ok(())
}
