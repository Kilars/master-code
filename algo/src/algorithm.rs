use crate::max_dtw::max_dtw;
use crate::plot::graph_trajectory;
use crate::rest::{EncodedTrajectory, Point, ReferenceList, SubTrajectory};
use crate::spatial_filter::PointWithIndexReference;

use rstar::RTree;
use serde::{de, Deserialize};
use std::collections::HashMap;

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
#[derive(Debug)]
pub struct Config {
    pub n: i32,
    pub rs: i32, //Reference set size in milliparts (thousandths)
    pub compression_ratio: i32,
    pub spatial_filter: bool,
    pub error_trajectories: i32,
    pub error_point: i32,
}
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub avg_cr: f64,
    pub avg_mdtw: f64,
    pub runtime: std::time::Duration,
}

fn reconstruct_compressed(compressed_trajectory: EncodedTrajectory) -> Vec<Point> {
    let mut reconstructed: Vec<Point> = Vec::new();
    compressed_trajectory
        .0
        .iter()
        .for_each(|compressed| match compressed {
            SubTrajectory::Reference(reference) => reference.iter().for_each(|point| {
                reconstructed.push(point.clone());
            }),
            SubTrajectory::Trajectory(raw_trajectory) => raw_trajectory.iter().for_each(|point| {
                reconstructed.push(point.clone());
            }),
        });

    reconstructed
}

pub fn rest_main(conf: Config) -> Result<PerformanceMetrics, csv::Error> {
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

    let begin_encoding = std::time::Instant::now();

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
    let mut index = 0;
    n_trajectories.iter().for_each(|t| {
        index += 1;
        let begin_encode = std::time::Instant::now();
        let (encoded, cr) = mrt_list.encode(
            &t,
            conf.error_trajectories as f64,
            conf.error_point as f64,
            r_tree.as_ref(),
        );
        let elapsed = begin_encode.elapsed();

        if elapsed > std::time::Duration::from_secs(60) {
            println!(
                "Percentage {:.2?}%, cr: {:.2}, time: {:.2?}, encoded len: {}",
                index as f64 / conf.n as f64 * 100.0,
                cr,
                elapsed,
                encoded.0.len()
            );
            let graph_res = graph_trajectory(
                &format!("./plots/encoded_{}.png", index),
                &format!("./plots/og_{}.png", index),
                encoded.clone(),
                t.clone(),
            );
            if graph_res.is_err() {
                println!("Error: {:?}", graph_res.err());
            } else {
                println!("Graph created");
            }
            //mrt_list.encode_with_debug_ts(
            //    &t,
            //    conf.error_trajectories as f64,
            //    conf.error_point as f64,
            //    r_tree.as_ref(),
            //);
        } else if index < 10 || (index > 1230 && index < 1240) {
            let _graph_res = graph_trajectory(
                &format!("./plots/encoded_{}.png", index),
                &format!("./plots/og_{}.png", index),
                encoded.clone(),
                t.clone(),
            );
        }
        encoded_trajectories.push((encoded, cr));
    });

    let runtime = begin_encoding.elapsed();

    println!("Runtime: {:.2?}", runtime);

    let performance_metrics_begin = std::time::Instant::now();

    let avg_cr = encoded_trajectories.iter().map(|(_, cr)| cr).sum::<f64>()
        / encoded_trajectories.len() as f64;

    println!("avg_cr {:.2?}", performance_metrics_begin.elapsed());
    let avg_mdtw = encoded_trajectories
        .iter()
        .enumerate()
        .map(|(index, (encoded, _))| {
            let mut map = HashMap::new();
            let original_trajectory = n_trajectories[index].clone();
            let reconstructed_from_encode = reconstruct_compressed(encoded.clone());
            let reconstructed_slice = reconstructed_from_encode.as_slice();

            let foo = max_dtw(&original_trajectory, reconstructed_slice, &mut map);

            foo
        })
        .sum::<f64>()
        / encoded_trajectories.len() as f64;

    println!("mdtw {:.2?}", performance_metrics_begin.elapsed());
    // Performance metrics:
    //  - Average Compression ratio
    //  - Average MaxDTW / Another error metric?
    //  - Runtime

    println!(
        "Performance metrics runtime {:.2?}",
        performance_metrics_begin.elapsed()
    );

    Ok(PerformanceMetrics {
        avg_cr,
        avg_mdtw,
        runtime,
    })
}
