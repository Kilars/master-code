use itertools::Itertools;
use rstar::RTree;
use serde::{de, Deserialize};

use crate::{
    rest::{encode, Point},
    spatial_filter::{PointWithIndexReference, SpatialQuery},
};

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
    let sample_to_build_reference_set: Vec<Vec<Point>> = csv::Reader::from_path("porto.csv")?
        .deserialize()
        .take(((conf.rs as f32 / 1000.0) * conf.n as f32) as usize)
        .map(|res| {
            res.map(|traj: CsvTrajectory| traj.polyline.iter().map(|&pnt| pnt.into()).collect_vec())
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("MRT source size: {}", sample_to_build_reference_set.len());
    let mut reference_set: Vec<Vec<Point>> = Vec::new();

    let mut r_tree: Option<RTree<PointWithIndexReference>> = if conf.spatial_filter {
        Some(RTree::<PointWithIndexReference>::new())
    } else {
        None
    };
    let begin_mrt = std::time::Instant::now();

    sample_to_build_reference_set.into_iter().for_each(|t| {
        let (_, compression_ratio) = encode(
            reference_set
                .iter()
                .map(|t| t.as_slice())
                .collect_vec()
                .as_slice(),
            &t.as_slice(),
            conf.error_trajectories as f64,
            conf.dtw_band,
            r_tree.as_ref(),
            conf.error_point as f64,
        );
        if compression_ratio < conf.compression_ratio as f64 {
            if let Some(mut_tree) = r_tree.as_mut() {
                for (i, point) in t.iter().enumerate() {
                    mut_tree.insert(PointWithIndexReference {
                        point: point.clone(),
                        index: (reference_set.len(), i),
                    });
                }
            }
            reference_set.push(t);
        }
    });

    println!("MRT list size: {}", reference_set.len());
    println!("MRT time: {:.2?}", begin_mrt.elapsed());

    let n_trajectories: Vec<Vec<Point>> = csv::Reader::from_path("porto.csv")?
        .deserialize()
        .skip(2)
        .take(1)
        .map(|res| {
            res.map(|traj: CsvTrajectory| {
                traj.polyline
                    .iter()
                    .map(|&pnt| pnt.into())
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut encoded_cr = Vec::new();

    n_trajectories.iter().for_each(|t| {
        let candidate_trajectories: Vec<&[Point]> = match &mut r_tree {
            Some(tree) => tree
                .points_within_envelope(conf.error_point as f64, t[0].clone())
                .iter()
                .map(|PointWithIndexReference { index: (i, j), .. }| &reference_set[*i][*j..])
                .collect_vec(),
            None => reference_set.iter().map(|t| t.as_slice()).collect_vec(),
        };
        println!("candidate_trajectories {}", candidate_trajectories.len());
        let (_, cr) = encode(
            reference_set
                .iter()
                .map(|t| t.as_slice())
                .collect_vec()
                .as_slice(),
            &t,
            conf.error_trajectories as f64,
            conf.dtw_band,
            r_tree.as_ref(),
            conf.error_point as f64,
        );

        encoded_cr.push(cr);
    });

    let runtime = begin.elapsed();

    let avg_cr = encoded_cr.iter().sum::<f64>() / encoded_cr.len() as f64;

    println!("Reference set size: {}", reference_set.len());
    Ok(PerformanceMetrics {
        avg_cr,
        avg_mdtw: 6.9,
        set_size: reference_set.len() as i32,
        runtime,
    })
}
