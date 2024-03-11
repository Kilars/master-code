use crate::rest::ReferenceList;
use crate::spatial_filter::sequential_mrt_build_spatial_filter;
use serde::{de, Deserialize};
pub mod max_dtw;
pub mod rest;
pub mod spatial_filter;

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
        .take(10000)
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

    sequential_mrt_build_spatial_filter(par_records);

    let mrt_list = ReferenceList {
        trajectories: Vec::new(),
    };

    //par_records.into_iter().for_each(|t| {
    //    let (_encoded_t, compression_ratio) = mrt_list.encode(&t, 0.2, None);
    //    if compression_ratio < 5.0 {
    //        mrt_list.trajectories.push(t);
    //    }
    //});
    let elapsed = begin.elapsed();
    println!("Reference set size: {:?}", mrt_list.trajectories.len());
    println!("duration {:.2?}", elapsed);
    Ok(())
}
