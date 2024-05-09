use crate::dtw_band::{dtw as dtw_normal, dtw_band};
use crate::spatial_filter::{PointWithIndexReference, SpatialQuery};
use haversine::{distance, Location};
use itertools::Itertools;
use rstar::RTree;
use std::collections::{HashMap, HashSet};

extern crate haversine;

impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Point {
        Point {
            lat: (value.0 * 1000000.0) as i32,
            lng: (value.1 * 1000000.0) as i32,
        }
    }
}
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Point {
    pub lat: i32,
    pub lng: i32,
}
impl Point {
    pub fn distance(&self, other: &Point) -> f64 {
        self.haversine(other)
    }
    pub fn haversine(&self, other: &Point) -> f64 {
        distance(
            Location {
                latitude: self.lat as f64 / 1000000.0,
                longitude: self.lng as f64 / 1000000.0,
            },
            Location {
                latitude: other.lat as f64 / 1000000.0,
                longitude: other.lng as f64 / 1000000.0,
            },
            haversine::Units::Kilometers,
        )
    }
    pub fn euclidean(&self, other: &Point) -> f32 {
        let dx = self.lat as f32 / 1000000.0 - other.lat as f32 / 1000000.0;
        let dy = self.lng as f32 / 1000000.0 - other.lng as f32 / 1000000.0;
        (dx.powf(2.0) + dy.powf(2.0)).sqrt()
    }
    pub fn lng_as_f32(&self) -> f32 {
        self.lng as f32 / 1000000.0
    }
    pub fn lat_as_f32(&self) -> f32 {
        self.lat as f32 / 1000000.0
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SubTrajectory<'a> {
    Trajectory(Vec<Point>),
    Reference(&'a [Point]),
}
#[derive(Clone)]
pub struct EncodedTrajectory<'a>(pub Vec<SubTrajectory<'a>>);

pub fn max_dtw<'a>(st: &'a [Point], rt: &'a [Point], band: usize) -> f64 {
    if band == 0 {
        return dtw_normal(st, rt);
    }
    dtw_band(st, rt, band)
}

pub fn encode<'a>(
    reference_trajectories: &'a [&[Point]],
    trajectory: &[Point],
    spatial_deviation: f64,
    band: usize,
    r_tree: Option<&RTree<PointWithIndexReference>>,
    spatial_filter_distance: f64,
) -> (EncodedTrajectory<'a>, f64) {
    let length = trajectory.len();
    let mut encoded_trajectory = EncodedTrajectory(Vec::new());
    let mut last_indexed_point = 0;
    let mut references = 0;
    let mut direct_points = 0;

    while last_indexed_point < length - 1 {
        let candidate_vector = match r_tree {
            Some(tree) => tree
                .points_within_envelope(
                    spatial_filter_distance,
                    trajectory[last_indexed_point].clone(),
                )
                .iter()
                .map(|PointWithIndexReference { index: (i, j), .. }| {
                    &reference_trajectories[*i][*j..]
                })
                .collect_vec(),
            None => reference_trajectories.to_vec(),
        };

        //spatial deviation from m to k
        match greedy_mrt_search(
            &trajectory[last_indexed_point..],
            candidate_vector.as_slice(),
            spatial_deviation / 1000.0,
            band,
        ) {
            Some((new_last_index, mrt)) => {
                last_indexed_point += new_last_index;
                encoded_trajectory.0.push(SubTrajectory::Reference(mrt));
                references += 1;
            }
            None => {
                encoded_trajectory.0.push(SubTrajectory::Trajectory(
                    trajectory[last_indexed_point..=last_indexed_point + 1].to_vec(),
                ));
                last_indexed_point += 1;
                direct_points += 1;
            }
        }
    }

    // i32 is 4 bytes
    let point_size = 4.0;
    // 8 byte reference
    let reference_size = 8.0;
    if direct_points > 0 {
        direct_points += 1;
    }

    let compression_ratio = (length as f64 * point_size)
        / ((direct_points as f64 * point_size) + (references as f64 * reference_size));

    (encoded_trajectory, compression_ratio)
}

fn greedy_mrt_search<'a>(
    trajectory: &[Point],
    reference_trajectories: &[&'a [Point]],
    max_deviation: f64,
    dtw_band: usize,
) -> Option<(usize, &'a [Point])> {
    let mut subtraj_mrt_map = HashMap::new();

    for reference_trajectory in reference_trajectories {
        let mut current_mrts: HashSet<(usize, usize)> = (0..reference_trajectory.len() - 1)
            .into_iter()
            .filter(|&j| {
                max_dtw(
                    &trajectory[0..=1],
                    &reference_trajectory[j..=j + 1],
                    dtw_band,
                ) < max_deviation
            })
            .map(|j| (j, j + 1))
            .collect();

        let mut trajectory_index = 1;
        while !current_mrts.is_empty() {
            trajectory_index += 1;
            current_mrts.iter().next().map(|arbitrary_match| {
                subtraj_mrt_map
                    .entry(trajectory_index)
                    .or_insert_with(|| &reference_trajectory[arbitrary_match.0..=arbitrary_match.1])
            });
            current_mrts = current_mrts
                .into_iter()
                .filter(|&(_, rt_end)| {
                    (trajectory_index < trajectory.len() - 1)
                        && (rt_end < reference_trajectory.len() - 1)
                })
                .flat_map(|(rt_start, rt_end)| {
                    [
                        (rt_start, rt_end),
                        (rt_end, rt_end + 1),
                        (rt_start, rt_end + 1),
                    ]
                    .into_iter()
                    .filter(|&(s, e)| {
                        max_dtw(
                            &trajectory[..=trajectory_index],
                            &reference_trajectory[s..=e],
                            dtw_band,
                        ) < max_deviation
                    })
                })
                .collect();
        }
    }

    subtraj_mrt_map.into_iter().max_by_key(|&(k, _)| k)
}
