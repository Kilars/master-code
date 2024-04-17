use crate::rest::ReferenceList;
use crate::spatial_filter::PointWithIndexReference;
use std::io::{self, Write};

use rstar::RTree;
use serde::{de, Deserialize};

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
#[derive(Debug, Clone)]
pub struct Config {
    pub n: i32,
    pub rs: i32, //Reference set size in milliparts (thousandths)
    pub compression_ratio: i32,
    pub spatial_filter: bool,
    pub dtw_band: usize,
    pub error_trajectories: i32,
    pub error_point: i32,
}
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub avg_cr: f64,
    pub avg_mdtw: f64,
    pub set_size: i32,
    pub runtime: std::time::Duration,
}

pub fn rest_main(conf: Config) -> Result<PerformanceMetrics, csv::Error> {
    let begin = std::time::Instant::now();
    let mrt_source = csv::Reader::from_path("porto.csv")?
        .deserialize()
        .take(((conf.rs as f32 / 1000.0) * conf.n as f32) as usize)
        .map(|res| {
            res.map(|traj: CsvTrajectory| {
                traj.polyline
                    .iter()
                    .map(|&pnt| pnt.into())
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("MRT source size: {}", mrt_source.len());
    let mut mrt_list = ReferenceList {
        trajectories: Vec::new(),
    };
    let mut r_tree: Option<RTree<PointWithIndexReference>> = if conf.spatial_filter {
        Some(RTree::new())
    } else {
        None
    };
    let begin_mrt = std::time::Instant::now();
    mrt_source.into_iter().for_each(|t| {
        let (_, compression_ratio) = mrt_list.encode(
            &t,
            conf.error_trajectories as f64,
            conf.dtw_band,
            conf.error_point as f64,
            r_tree.as_ref(),
        );
        if compression_ratio < conf.compression_ratio as f64 {
            if let Some(mut_tree) = r_tree.as_mut() {
                for (i, point) in t.iter().enumerate() {
                    mut_tree.insert(PointWithIndexReference {
                        point: point.clone(),
                        index: (mrt_list.trajectories.len(), i),
                    });
                }
            }
            mrt_list.trajectories.push(t);
        }
    });

    println!("MRT list size: {}", mrt_list.trajectories.len());
    println!("MRT time: {:.2?}", begin_mrt.elapsed());

    let n_trajectories = csv::Reader::from_path("porto.csv")?
        .deserialize()
        .take(conf.n as usize)
        .map(|res| {
            res.map(|traj: CsvTrajectory| {
                traj.polyline
                    .iter()
                    .map(|&pnt| pnt.into())
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut encoded_trajectories = Vec::new();
    n_trajectories.iter().for_each(|t| {
        io::stdout().flush().unwrap();
        let (encoded, cr) = mrt_list.encode(
            &t,
            conf.error_trajectories as f64,
            conf.dtw_band,
            conf.error_point as f64,
            r_tree.as_ref(),
        );

        encoded_trajectories.push((encoded, cr));
    });

    let runtime = begin.elapsed();

    let avg_cr = encoded_trajectories.iter().map(|(_, cr)| cr).sum::<f64>()
        / encoded_trajectories.len() as f64;

    Ok(PerformanceMetrics {
        avg_cr,
        avg_mdtw: 6.9,
        set_size: mrt_list.trajectories.len() as i32,
        runtime,
    })
}
