use itertools::Itertools;

use std::io::{self, Write};

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
struct RestMode {
    pub mode_name: String,
    pub rs: i32, //Reference set size in milliparts (thousandths)
    pub compression_ratio: i32,
    pub spatial_filter: bool,
    pub dtw_band: usize,
    pub error_point: i32,
}
#[derive(Debug, Clone)]
struct DpMode {
    pub mode_name: String,
}
#[derive(Debug, Clone)]
enum Mode {
    Rest(RestMode),
    DP(DpMode),
}
#[derive(Debug, Clone)]
pub struct Config {
    pub n: i32,
    pub max_dtw_dist: i32,
    pub mode: Mode,
}
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub avg_cr: f64,
    pub set_size: i32,
    pub runtime: std::time::Duration,
}

pub fn rest_main(conf: Config, only_set: bool) -> Result<PerformanceMetrics, csv::Error> {
    let begin = std::time::Instant::now();
    match conf.mode {
        Mode::Rest(rest_conf) => {
            let sample_to_build_reference_set: Vec<Vec<Point>> =
                csv::Reader::from_path("porto.csv")?
                    .deserialize()
                    .take(((rest_conf.rs as f32 / 1000.0) * conf.n as f32) as usize)
                    .map(|res| {
                        res.map(|traj: CsvTrajectory| {
                            traj.polyline.iter().map(|&pnt| pnt.into()).collect_vec()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

            let mut reference_set: Vec<Vec<Point>> = Vec::new();
            let mut r_tree: Option<RTree<PointWithIndexReference>> = if rest_conf.spatial_filter {
                Some(RTree::<PointWithIndexReference>::new())
            } else {
                None
            };
            println!("MRT list size: {}", reference_set.len());
            let begin_mrt = std::time::Instant::now();

            let mut file = std::fs::File::options()
                .create(true)
                .append(true)
                .open("out/set_size.txt")
                .expect("Failed to open or create the file");

            sample_to_build_reference_set
                .into_iter()
                .enumerate()
                .for_each(|(i, t)| {
                    let reference_vec = reference_set.iter().map(|t| t.as_slice()).collect_vec();
                    let (_, compression_ratio) = encode(
                        reference_vec.as_slice(),
                        &t.as_slice(),
                        conf.max_dtw_dist as f64,
                        rest_conf.dtw_band,
                        r_tree.as_ref(),
                        rest_conf.error_point as f64,
                    );

                    if compression_ratio < rest_conf.compression_ratio as f64 {
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

                    if i % 5000 == 0 {
                        let _file_write_res = write!(
                            file,
                            "{},{},{},{},{},{},\n",
                            i,
                            reference_set.len(),
                            rest_conf.spatial_filter,
                            conf.max_dtw_dist,
                            rest_conf.error_point,
                            begin_mrt.elapsed().as_secs_f64(),
                        );
                    }
                });

            println!("MRT time: {:.2?}", begin_mrt.elapsed());
            println!("Reference set size: {}", reference_set.len());
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
            let final_reference_vectors = reference_set.iter().map(|t| t.as_slice()).collect_vec();
            n_trajectories.iter().for_each(|t| {
                let (encoded_trajectory, compression_ratio) = encode(
                    final_reference_vectors.as_slice(),
                    &t.as_slice(),
                    conf.max_dtw_dist as f64,
                    rest_conf.dtw_band,
                    r_tree.as_ref(),
                    rest_conf.error_point as f64,
                );
                encoded_cr.push((encoded_trajectory, compression_ratio));
            });
            let runtime = begin.elapsed();
            let avg_cr = encoded_cr.iter().map(|(_, cr)| cr).sum::<f64>() / encoded_cr.len() as f64;

            Ok(PerformanceMetrics {
                avg_cr,
                set_size: reference_set.len() as i32,
                runtime,
            })
        }
        Mode::DP(dp_rest_conf) => {
            todo!()
        }
    }
}
