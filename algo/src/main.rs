use crate::rest::ReferenceSet;
use itertools::Itertools;
use serde::{de, Deserialize};
use std::collections::HashSet;
pub mod max_dtw;
pub mod rest;

#[derive(Deserialize, Clone)]
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
    let begin = std::time::Instant::now();
    let csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;

    let mut mrt_set = ReferenceSet(HashSet::new());

    // Generate reference set
    let (l, r) = csv_trajectories.split_at(13);
    let _split = l.iter().chain(&r[1..]).cloned();
    for traj in csv_trajectories {
        let sub_start = std::time::Instant::now();
        let t = traj
            .polyline
            .iter()
            .map(|&pnt| pnt.into())
            .collect::<Vec<_>>();
        println!(
            "begin encoding trajectory: {:?} with {:?} in mrt set",
            t.len(),
            mrt_set.0.len()
        );
        let (_encoded_t, compression_ratio) = mrt_set.encode(&t, 0.2);
        println!("{:?}", compression_ratio);
        if compression_ratio < 5.0 {
            println!("inserting length: {:?} to mrt_set \n", t.len());
            mrt_set.0.insert(t);
        }
        println!("duration: {:.2?}", sub_start.elapsed());
    }
    let elapsed = begin.elapsed();
    println!("Reference set size: {:?}", mrt_set.0.len());
    println!("duration {:.2?}", elapsed);
    Ok(())
}
