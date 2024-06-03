use itertools::Itertools;

use std::io::{self, Write};

use rstar::RTree;
use serde::{de, Deserialize};

use crate::{
    dp::douglas_peucker,
    rest::{encode, Point, SubTrajectory},
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
#[derive(Debug, Clone, Copy)]
pub struct RestMode {
    pub rs: i32, //Reference set size in milliparts (thousandths)
    pub compression_ratio: i32,
    pub spatial_filter: bool,
    pub include_entire_trajectory: bool,
    pub k: usize,
    pub error_point: i32,
}
#[derive(Debug, Clone)]
pub struct DpMode {}
#[derive(Debug, Clone)]
pub enum Mode {
    Rest(RestMode),
    DP(DpMode),
}
#[derive(Debug, Clone)]
pub struct Config {
    pub n: i32,
    pub max_dtw_dist: i32,
    pub dtw_band: usize,
    pub mode: Mode,
}
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub avg_cr: f64,
    pub max_dtw_dist: i32,
    pub set_size: i32,
    pub runtime: std::time::Duration,
}

pub fn cr_from_shape(shape: (u64, u64, u64)) -> f64 {
    // i32 is 4 bytes, and x2 for lat and lng
    let point_size = 4.0 * 2.0;
    // 8 byte reference
    let reference_size = 8.0;
    (shape.0 as f64 * point_size)
        / ((shape.2 as f64 * point_size) + (shape.1 as f64 * reference_size))
}
pub fn rest_main(
    conf: Config,
    only_set: bool,
    log_n: i32,
) -> Result<PerformanceMetrics, csv::Error> {
    let mut set_size_file = std::fs::File::options()
        .create(true)
        .append(true)
        .open("out/set_size.txt")
        .expect("Failed to open or create the file");
    let mut intermediate_file = std::fs::File::options()
        .create(true)
        .append(true)
        .open("out/intermediate.txt")
        .expect("Failed to open or create the file");
    let begin = std::time::Instant::now();
    let mut compressed_points = 0;
    let mut references = 0;
    let mut raw_points = 0;
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

            sample_to_build_reference_set
                .into_iter()
                .enumerate()
                .for_each(|(i, t)| {
                    let reference_vec = reference_set.iter().map(|t| t.as_slice()).collect_vec();
                    let (encoded, shape) = encode(
                        reference_vec.as_slice(),
                        &t.as_slice(),
                        conf.max_dtw_dist as f64,
                        conf.dtw_band,
                        rest_conf.k,
                        r_tree.as_ref(),
                        rest_conf.error_point as f64,
                    );

                    if cr_from_shape(shape) < rest_conf.compression_ratio as f64 {
                        if rest_conf.include_entire_trajectory {
                            if let Some(mut_tree) = r_tree.as_mut() {
                                for (i, point) in t.iter().enumerate() {
                                    mut_tree.insert(PointWithIndexReference {
                                        point: point.clone(),
                                        index: (reference_set.len(), i),
                                    });
                                }
                            }
                            reference_set.push(t);
                            raw_points += shape.0;
                        } else {
                            let mut raw_trajectories_added = Vec::new();
                            let mut first_point_index = 0;
                            for st in encoded.0 {
                                match &st {
                                    SubTrajectory::Trajectory(raw_trajectory) => {
                                        for p in raw_trajectory
                                            [first_point_index..raw_trajectory.len()]
                                            .iter()
                                        {
                                            raw_trajectories_added.push(Some(p.clone()));
                                        }
                                    }
                                    // Successfully compressed, therefore not added to reference set
                                    SubTrajectory::Reference(_) => {
                                        raw_trajectories_added.push(None);
                                    }
                                }
                                first_point_index = 1;
                            }
                            let mut current_batch = Vec::new();

                            for item in raw_trajectories_added.iter() {
                                match item {
                                    Some(p) => current_batch.push(p.clone()),
                                    None => {
                                        if !current_batch.is_empty() {
                                            raw_points += current_batch.len() as u64;
                                            if let Some(mut_tree) = r_tree.as_mut() {
                                                for (i, point) in current_batch.iter().enumerate() {
                                                    mut_tree.insert(PointWithIndexReference {
                                                        point: point.clone(),
                                                        index: (reference_set.len(), i),
                                                    });
                                                }
                                            }
                                            reference_set.push(current_batch.clone()); // Push the current batch as one element
                                            current_batch.clear(); // Reset the current batch
                                        }
                                    }
                                }
                            }
                            if !current_batch.is_empty() {
                                raw_points += current_batch.len() as u64;
                                if let Some(mut_tree) = r_tree.as_mut() {
                                    for (i, point) in current_batch.iter().enumerate() {
                                        mut_tree.insert(PointWithIndexReference {
                                            point: point.clone(),
                                            index: (reference_set.len(), i),
                                        });
                                    }
                                }

                                reference_set.push(current_batch.clone()); // Push the current batch as one element
                                current_batch.clear(); // Reset the current batch
                            }
                        }
                    }

                    if (i + 1) as i32
                        % ((((rest_conf.rs as f32 / 1000.0) * conf.n as f32) as i32) / 5)
                        == 0
                    {
                        let _file_write_res = write!(
                            set_size_file,
                            "{},{},{},{},{},{}\n",
                            match conf.mode.clone() {
                                Mode::Rest(rest_conf) => {
                                    let mut mode_name = String::from("REST"); // Change to mutable String
                                    if rest_conf.spatial_filter {
                                        mode_name.push_str("-SF"); // Use push_str to append
                                        mode_name.push_str(&rest_conf.error_point.to_string());
                                        // Convert error_point to String and append
                                    }
                                    if conf.dtw_band != 0 {
                                        mode_name.push_str("-BND"); // Append "-BND"
                                        mode_name.push_str(&conf.dtw_band.to_string());
                                        // Convert dtw_band to String and append
                                    }
                                    mode_name
                                }
                                Mode::DP(_) => {
                                    let mut mode_name = String::from("DP");
                                    if conf.dtw_band != 0 {
                                        mode_name.push_str("-BND"); // Append "-BND"
                                        mode_name.push_str(&conf.dtw_band.to_string());
                                        // Convert dtw_band to String and append
                                    }
                                    mode_name
                                }
                            },
                            conf.max_dtw_dist,
                            i + 1,
                            format!(
                                "{:.2}",
                                ((rest_conf.rs as f32 / 1000.0) * conf.n as f32) as usize
                            ),
                            reference_set.len(),
                            format!("{:.0}", begin.elapsed().as_secs_f64()),
                        );
                    }
                });
            if only_set {
                return Ok(PerformanceMetrics {
                    avg_cr: 0.0,
                    set_size: reference_set.len() as i32,
                    max_dtw_dist: conf.max_dtw_dist,
                    runtime: begin.elapsed(),
                });
            }

            let n_trajectories: Vec<Vec<Point>> = csv::Reader::from_path("porto.csv")?
                .deserialize()
                .skip(((rest_conf.rs as f32 / 1000.0) * conf.n as f32) as usize)
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
            n_trajectories.iter().enumerate().for_each(|(i, t)| {
                let (encoded_trajectory, shape) = encode(
                    final_reference_vectors.as_slice(),
                    &t.as_slice(),
                    conf.max_dtw_dist as f64,
                    conf.dtw_band,
                    rest_conf.k,
                    r_tree.as_ref(),
                    rest_conf.error_point as f64,
                );
                encoded_cr.push((encoded_trajectory, shape));
                compressed_points += shape.0;
                references += shape.1;
                raw_points += shape.2;

                if (i + 1) as i32 % (conf.n / log_n) == 0 {
                    let avg_cr = encoded_cr
                        .iter()
                        .map(|&(_, shape)| cr_from_shape(shape))
                        .sum::<f64>()
                        / encoded_cr.len() as f64;
                    let cr_set_inclusive =
                        cr_from_shape((compressed_points, references, raw_points));
                    let _file_write_res = write!(
                        intermediate_file,
                        "{},{},{},{},{},{},{},{}\n",
                        match conf.mode.clone() {
                            Mode::Rest(rest_conf) => {
                                let mut mode_name = String::from("REST"); // Change to mutable String
                                if !rest_conf.include_entire_trajectory {
                                    mode_name.push_str("_EXCL");
                                }
                                if rest_conf.spatial_filter {
                                    mode_name.push_str("-SF"); // Use push_str to append
                                    mode_name.push_str(&rest_conf.error_point.to_string());
                                    // Convert error_point to String and append
                                }
                                if conf.dtw_band != 0 {
                                    mode_name.push_str("-BND"); // Append "-BND"
                                    mode_name.push_str(&conf.dtw_band.to_string());
                                    // Convert dtw_band to String and append
                                }
                                if rest_conf.k != 0 {
                                    mode_name.push_str("-KNN");
                                    mode_name.push_str(&rest_conf.k.to_string());
                                }
                                mode_name
                            }
                            Mode::DP(_) => {
                                let mut mode_name = String::from("DP");
                                if conf.dtw_band != 0 {
                                    mode_name.push_str("-BND"); // Append "-BND"
                                    mode_name.push_str(&conf.dtw_band.to_string());
                                    // Convert dtw_band to String and append
                                }
                                mode_name
                            }
                        },
                        i + 1,
                        conf.max_dtw_dist,
                        format!(
                            "{:.2}",
                            ((rest_conf.rs as f32 / 1000.0) * conf.n as f32) as usize
                        ),
                        rest_conf.k,
                        format!("{:.0}", begin.elapsed().as_secs_f64()),
                        format!("{:.2}", avg_cr),
                        format!("{:.2}", cr_set_inclusive),
                    );
                }
            });
            let avg_cr = encoded_cr
                .iter()
                .map(|&(_, shape)| cr_from_shape(shape))
                .sum::<f64>()
                / encoded_cr.len() as f64;

            Ok(PerformanceMetrics {
                avg_cr,
                set_size: reference_set.len() as i32,
                max_dtw_dist: conf.max_dtw_dist,
                runtime: begin.elapsed(),
            })
        }
        Mode::DP(_) => {
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
            n_trajectories.iter().enumerate().for_each(|(i, t)| {
                let encoded_trajectory = douglas_peucker(
                    t.as_slice(),
                    conf.max_dtw_dist as f64 / 1000.0,
                    conf.dtw_band,
                );
                let cr = t.len() as f64 / encoded_trajectory.len() as f64;
                if (i + 1) as i32 % (conf.n / log_n) == 0 {
                    let avg_cr =
                        encoded_cr.iter().map(|(_, cr)| cr).sum::<f64>() / encoded_cr.len() as f64;
                    let _file_write_res = write!(
                        intermediate_file,
                        "{},{},{},{},{},{},{},{}\n",
                        match conf.mode.clone() {
                            Mode::Rest(rest_conf) => {
                                let mut mode_name = String::from("REST"); // Change to mutable String
                                if !rest_conf.include_entire_trajectory {
                                    mode_name.push_str("_EXCL");
                                }
                                if rest_conf.spatial_filter {
                                    mode_name.push_str("-SF"); // Use push_str to append
                                    mode_name.push_str(&rest_conf.error_point.to_string());
                                    // Convert error_point to String and append
                                }
                                if conf.dtw_band != 0 {
                                    mode_name.push_str("-BND"); // Append "-BND"
                                    mode_name.push_str(&conf.dtw_band.to_string());
                                    // Convert dtw_band to String and append
                                }
                                if rest_conf.k != 0 {
                                    mode_name.push_str("-KNN");
                                    mode_name.push_str(&rest_conf.k.to_string());
                                }
                                mode_name
                            }
                            Mode::DP(_) => {
                                let mut mode_name = String::from("DP");
                                if conf.dtw_band != 0 {
                                    mode_name.push_str("-BND"); // Append "-BND"
                                    mode_name.push_str(&conf.dtw_band.to_string());
                                    // Convert dtw_band to String and append
                                }
                                mode_name
                            }
                        },
                        i + 1,
                        conf.max_dtw_dist,
                        0,
                        0,
                        format!("{:.0}", begin.elapsed().as_secs_f64()),
                        format!("{:.2}", avg_cr),
                        //avg_cr = set_inclusive_cr for DP because there is no overhead
                        format!("{:.2}", avg_cr),
                    );
                }
                encoded_cr.push((encoded_trajectory, cr));
            });
            let avg_cr = encoded_cr.iter().map(|(_, cr)| cr).sum::<f64>() / encoded_cr.len() as f64;
            Ok(PerformanceMetrics {
                avg_cr,
                set_size: 0,
                max_dtw_dist: conf.max_dtw_dist,
                runtime: begin.elapsed(),
            })
        }
    }
}
