use crate::dtw_band::{dtw as dtw_normal, dtw_band};
use haversine::{distance, Location};
use itertools::Itertools;
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
    candidate_reference_trajectories: &'a [&[Point]],
    trajectory: &[Point],
    spatial_deviation: f64,
    band: usize,
) -> (EncodedTrajectory<'a>, f64) {
    let length = trajectory.len();
    let mut encoded_trajectory = EncodedTrajectory(Vec::new());
    let mut last_indexed_point = 0;
    let mut references = 0;
    let mut direct_points = 0;

    while last_indexed_point < length - 1 {
        //spatial deviation from m to k
        match greedy_mrt_expand(
            candidate_reference_trajectories,
            &trajectory[last_indexed_point..],
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

fn greedy_mrt_expand<'a>(
    candidate_reference_trajectories: &'a [&[Point]],
    st: &[Point],
    spatial_deviation: f64,
    band: usize,
) -> Option<(usize, &'a [Point])> {
    let mut length_match_map = HashMap::new();
    for rt in candidate_reference_trajectories {
        let mut current_rt_matches: HashSet<(usize, usize)> = (0..rt.len() - 1)
            .into_iter()
            .filter(|&j| max_dtw(&st[0..=1], &rt[j..=j + 1], band) < spatial_deviation)
            .map(|j| (j, j + 1))
            .collect();

        let mut matched_st_len = 1;
        while !current_rt_matches.is_empty() {
            matched_st_len += 1;
            let expanded_matches: HashSet<(usize, usize)> = current_rt_matches
                .iter()
                .filter(|&(_, rt_end)| (matched_st_len < st.len() - 1) && (*rt_end < rt.len() - 1))
                .map(|&(rt_start, rt_end)| {
                    [
                        (rt_start, rt_end),
                        (rt_end, rt_end + 1),
                        (rt_start, rt_end + 1),
                    ]
                    .iter()
                    .filter(|&&(s, e)| {
                        max_dtw(&st[..=matched_st_len], &rt[s..=e], band) < spatial_deviation
                    })
                    .copied()
                    .collect_vec()
                })
                .flatten()
                .collect();

            if expanded_matches.is_empty() {
                // if no match of the current length is found, insert an arbitrary match
                length_match_map.entry(matched_st_len).or_insert({
                    let arbitrary_match = current_rt_matches.iter().next().unwrap();
                    &rt[arbitrary_match.0..=arbitrary_match.1]
                });
                break;
            } else {
                current_rt_matches = expanded_matches;
            }
        }
    }

    length_match_map
        .into_iter()
        .max_by_key(|&(k, _)| k)
        .filter(|&(k, _)| k != 0)
}
