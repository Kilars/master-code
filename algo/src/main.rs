use crate::rest::ReferenceList;
use rayon::prelude::*;
use serde::{de, Deserialize};
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
    let par_records = csv::Reader::from_path("porto.csv")?
        .deserialize()
        .map(|res| {
            res.map(|traj: CsvTrajectory| {
                traj.polyline
                    .iter()
                    .map(|&pnt| pnt.into())
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("csv in memory {:.2?}", begin.elapsed());

    let mut mrt_list = ReferenceList(Vec::new());

    par_records.into_iter().for_each(|t| {
        let (_encoded_t, compression_ratio) = mrt_list.encode(&t, 0.2);
        if compression_ratio < 5.0 {
            mrt_list.0.push(t);
        }
    });
    let elapsed = begin.elapsed();
    println!("Reference set size: {:?}", mrt_list.0.len());
    println!("duration {:.2?}", elapsed);
    Ok(())
}
