use itertools::Itertools;

use std::io::{self, Write};

use rstar::RTree;
use serde::{de, Deserialize};

use crate::{
    plot::graph_trajectory,
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
    pub set_size: i32,
    pub runtime: std::time::Duration,
}

pub fn rest_main(conf: Config, only_set: bool) -> Result<PerformanceMetrics, csv::Error> {
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

    let mut i = 0;
    let mut runtime_file = std::fs::File::options()
        .create(true)
        .append(true)
        .open("out/runtime_index.txt")
        .expect("Failed to open or create the file");
    let mut file = std::fs::File::options()
        .create(true)
        .append(true)
        .open("out/set_size.txt")
        .expect("Failed to open or create the file");

    sample_to_build_reference_set.into_iter().for_each(|t| {
        let length = t.len();
        let candidate_vectors = match r_tree.clone() {
            Some(tree) => tree
                .points_within_envelope(conf.error_point as f64, t[0].clone())
                .iter()
                .map(|PointWithIndexReference { index: (i, j), .. }| &reference_set[*i][*j..])
                .collect_vec()
                .len(),
            None => reference_set
                .iter()
                .map(|t| t.as_slice())
                .collect_vec()
                .len(),
        };

        let begin_local = std::time::Instant::now();

        let reference_vec = reference_set.iter().map(|t| t.as_slice()).collect_vec();
        let (_, compression_ratio) = encode(
            reference_vec.as_slice(),
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

        // let _ = write!(
        //     runtime_file,
        //     "{},{:.2},{},{},{},\n",
        //     i,
        //     begin_mrt.elapsed().as_secs_f64() / 60.0,
        //     length,
        //     candidate_vectors,
        //     begin_local.elapsed().as_secs_f64()
        // );
        i += 1;
        if i % 5000 == 0 {
            let _file_write_res = write!(
                file,
                "{},{},{},{},{},{},\n",
                i,
                reference_set.len(),
                conf.spatial_filter,
                conf.error_trajectories,
                conf.error_point,
                begin_mrt.elapsed().as_secs_f64(),
            );
        }
        // print!("\r {}", i);
        // std::io::stdout().flush().unwrap();
    });

    println!("MRT list size: {}", reference_set.len());
    println!("MRT time: {:.2?}", begin_mrt.elapsed());

    if only_set {
        return Ok(PerformanceMetrics {
            avg_cr: 0.0,
            set_size: reference_set.len() as i32,
            runtime: begin.elapsed(),
        });
    }

    let n_trajectories: Vec<Vec<Point>> = csv::Reader::from_path("porto.csv")?
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

    let mut encoded_cr = Vec::new();

    println!("b4 final reference vectors");
    let final_reference_vectors = reference_set.iter().map(|t| t.as_slice()).collect_vec();
    let mut index = 0;
    n_trajectories.iter().for_each(|t| {
        print!("\r{} len: {}", index, t.len());
        let (encoded_trajectory, compression_ratio) = encode(
            final_reference_vectors.as_slice(),
            &t.as_slice(),
            conf.error_trajectories as f64,
            conf.dtw_band,
            r_tree.as_ref(),
            conf.error_point as f64,
        );

        std::io::stdout().flush().unwrap();
        if index > 5000 && index < 5030 {
            let _ = graph_trajectory(
                format!("plots/new_{}.png", index),
                encoded_trajectory.clone(),
                t.to_vec(),
            );
        }

        encoded_cr.push((encoded_trajectory, compression_ratio));
        index += 1;
    });

    let runtime = begin.elapsed();

    let avg_cr = encoded_cr.iter().map(|(_, cr)| cr).sum::<f64>() / encoded_cr.len() as f64;

    println!("Reference set size: {}", reference_set.len());
    Ok(PerformanceMetrics {
        avg_cr,
        set_size: reference_set.len() as i32,
        runtime,
    })
}
