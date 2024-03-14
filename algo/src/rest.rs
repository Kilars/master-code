use crate::max_dtw::max_dtw;
use crate::spatial_filter::PointWithIndexReference;

use haversine::{distance, Location};
use rstar::{RTree, AABB};
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
#[derive(PartialEq, Eq, Hash)]
pub enum SubTrajectory<'a> {
    Trajectory(Vec<Point>),
    Reference(&'a [Point]),
}
pub struct EncodedTrajectory<'a>(pub Vec<SubTrajectory<'a>>);
pub struct ReferenceList {
    pub trajectories: Vec<Vec<Point>>,
}

impl ReferenceList {
    pub fn encode(
        &self,
        trajectory: &Vec<Point>,
        spatial_deviation: f64,
        r_tree_point_threshold: f64,
        r_tree: Option<&RTree<PointWithIndexReference>>,
    ) -> (EncodedTrajectory, f64) {
        let length = trajectory.len();
        let mut encoded_trajectory = EncodedTrajectory(Vec::new());
        let mut last_indexed_point = 0;
        let mut references = 0;
        let mut direct_points = 0;
        while last_indexed_point < length - 1 {
            //spatial deviation from m to k
            match self.greedy_mrt_expand(
                &trajectory[last_indexed_point..],
                spatial_deviation / 1000.0,
                r_tree_point_threshold,
                r_tree,
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

    fn greedy_mrt_expand(
        &self,
        st: &[Point],
        spatial_deviation: f64,
        r_tree_point_threshold: f64,
        r_tree: Option<&RTree<PointWithIndexReference>>,
    ) -> Option<(usize, &[Point])> {
        let mut rt_match_map = HashMap::new();
        let spatial_filter: Vec<&[Point]> = match r_tree {
            Some(tree) => {
                let lat_d = (r_tree_point_threshold / 111319.9) as f32;
                let lng_d = r_tree_point_threshold as f32
                    / (111319.9 * st[0].lat_as_f32().to_radians().cos());
                tree.locate_in_envelope(&AABB::from_corners(
                    [
                        st[0].lat_as_f32() + lat_d as f32,
                        st[0].lng_as_f32() + lng_d as f32,
                    ],
                    [
                        st[0].lat_as_f32() - lat_d as f32,
                        st[0].lng_as_f32() - lng_d as f32,
                    ],
                ))
                .map(|x| &self.trajectories[x.index.0][x.index.1..])
                .collect::<Vec<_>>()
            }
            None => self.trajectories.iter().map(|x| &x[..]).collect::<Vec<_>>(),
        };
        for rt in spatial_filter {
            let mut memoization = HashMap::new();
            let mut st_last_match = 0;
            let mut rt_match = HashSet::new();
            for j in 0..rt.len() - 1 {
                if max_dtw(&st[..=st_last_match + 1], &rt[j..=j + 1], &mut memoization)
                    < spatial_deviation
                {
                    rt_match.insert((j, j + 1));
                }
            }
            while !rt_match.is_empty() {
                let mut to_insert = Vec::new();
                for m in rt_match.iter() {
                    let &(rt_start, rt_end) = m;
                    let st_index_check = st_last_match < st.len() - 1;
                    let rt_index_check = rt_end < rt.len() - 1;
                    if st_index_check && rt_index_check {
                        st_last_match += 1;
                        let local_st = &st[..=st_last_match];
                        let rta = &rt[rt_start..=rt_end];
                        let rtb = &rt[rt_end..=rt_end + 1];
                        let rtab = &rt[rt_start..=rt_end + 1];
                        if max_dtw(local_st, rta, &mut memoization) < spatial_deviation {
                            to_insert.push((rt_start, rt_end));
                        }
                        if max_dtw(local_st, rtb, &mut memoization) < spatial_deviation {
                            to_insert.push((rt_end, rt_end + 1));
                        }
                        if max_dtw(local_st, rtab, &mut memoization) < spatial_deviation {
                            to_insert.push((rt_start, rt_end + 1));
                        }
                    }
                }
                if to_insert.is_empty() {
                    let example_match = rt_match.iter().next().unwrap();
                    match rt_match_map.get(&st_last_match) {
                        Some(_) => {}
                        None => {
                            rt_match_map
                                .insert(st_last_match, &rt[example_match.0..=example_match.1]);
                        }
                    }
                    break;
                } else {
                    rt_match.clear();
                    for new_m in to_insert {
                        rt_match.insert(new_m);
                    }
                }
            }
        }
        match rt_match_map
            .iter()
            .filter(|&(&k, _)| k != 0)
            .max_by_key(|&(k, _)| k)
        {
            Some((&highest_key_entry, &value)) => Some((highest_key_entry, value)),
            None => None,
        }
    }
}
