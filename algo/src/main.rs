use crate::rest::ReferenceSet;
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
    println!("begin reading csv");
    let rdr = csv::Reader::from_path("porto.csv");

    let mut mrt_set = ReferenceSet(HashSet::new());

    // Generate reference set
    for (i, res) in rdr?.deserialize().enumerate() {
        println!("progress {:.2?}%", (i as f64 * 100.0) / 1600000.0);
        let traj: CsvTrajectory = res?;
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
