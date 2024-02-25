use crate::max_dtw::max_dtw;
extern crate haversine;
use haversine::{distance, Location};
use std::collections::{HashMap, HashSet};

impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Point {
        Point {
            lat: (value.0 * 1000000.0) as i32,
            lng: (value.1 * 1000000.0) as i32,
        }
    }
}
#[derive(Clone, Eq, PartialEq, Hash)]
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
}
#[derive(PartialEq, Eq, Hash)]
enum SubTrajectory<'a> {
    Trajectory(Vec<Point>),
    Reference(&'a [Point]),
}
#[derive(PartialEq, Eq, Hash)]
struct ReferenceSubTrajectory<'a>(&'a [Point]);

pub struct EncodedTrajectory<'a>(Vec<SubTrajectory<'a>>);
pub struct ReferenceSet(pub HashSet<Vec<Point>>);
struct SubTrajectoryReferenceIndex<'a>(
    HashMap<(usize, usize), HashSet<ReferenceSubTrajectory<'a>>>,
);

impl ReferenceSet {
    pub fn encode(
        &self,
        trajectory: &Vec<Point>,
        spatial_deviation: f64,
    ) -> (EncodedTrajectory, f32) {
        let mrt_map = self.get_subtrajectory_references(&trajectory, spatial_deviation);
        let length = trajectory.len();
        let mut references = 0;
        let mut direct_points = 0;

        let mut encoded_trajectory = EncodedTrajectory(Vec::new());
        let mut first_point_not_indexed = 0;
        let mut last_indexed_point = 0;
        let mut longest_mrt: Option<ReferenceSubTrajectory> = None;
        while first_point_not_indexed < length - 1 {
            match mrt_map
                .0
                .get(&(first_point_not_indexed, last_indexed_point + 1))
                .and_then(|mrt_set| mrt_set.iter().next())
            {
                Some(next_mrt) => {
                    last_indexed_point = last_indexed_point + 1;
                    longest_mrt = Some(ReferenceSubTrajectory(next_mrt.0));
                }
                None => {
                    if let Some(longest_mrt) = longest_mrt.take() {
                        encoded_trajectory
                            .0
                            .push(SubTrajectory::Reference(longest_mrt.0));
                        first_point_not_indexed = last_indexed_point + 1;
                        references += 1;
                    } else {
                        encoded_trajectory.0.push(SubTrajectory::Trajectory(
                            trajectory[first_point_not_indexed..last_indexed_point + 1].to_vec(),
                        ));
                        first_point_not_indexed = last_indexed_point + 1;
                        last_indexed_point += 1;
                        direct_points += 1;
                    }
                    longest_mrt = None;
                }
            }
        }
        if direct_points > 0 {
            direct_points += 1;
        }
        // i32 is 4 bytes
        let point_size = 4;
        // 8 byte reference
        let reference_size = 8;

        let compression_ratio = (length * point_size) as f32
            / (direct_points * point_size + references * reference_size) as f32;

        (encoded_trajectory, compression_ratio)
    }

    fn get_subtrajectory_references(
        &self,
        trajectory: &Vec<Point>,
        spatial_deviation: f64,
    ) -> SubTrajectoryReferenceIndex {
        let mut trajectory_mrt_dict: SubTrajectoryReferenceIndex =
            SubTrajectoryReferenceIndex(HashMap::new());

        for i in 0..trajectory.len() - 1 {
            let mut local_hash_set: HashSet<ReferenceSubTrajectory> = HashSet::new();
            for rt in self.0.iter() {
                for j in 0..rt.len() - 1 {
                    if max_dtw(&trajectory[i..=i + 1], &rt[j..=j + 1]) < spatial_deviation {
                        local_hash_set.insert(ReferenceSubTrajectory(&rt[j..=j + 1]));
                    }
                }
            }
            if local_hash_set.len() > 0 {
                trajectory_mrt_dict.0.insert((i, i + 1), local_hash_set);
            }
        }

        for length in 3..=trajectory.len() {
            for i in 0..trajectory.len() - length + 1 {
                let subtrajectory = &trajectory[i..i + length];
                let a_length = length - 1;
                let b_length = 2;
                let a_index = (i, i + a_length - 1);
                let b_index = (i + a_length, i + a_length + b_length - 1);
                let rtas = trajectory_mrt_dict.0.get(&a_index);
                let rtbs = trajectory_mrt_dict.0.get(&b_index);
                let mut local_hash_set: HashSet<ReferenceSubTrajectory> = HashSet::new();
                match (rtas, rtbs) {
                    (Some(rtas), Some(rtbs)) => {
                        for rta in rtas {
                            for rtb in rtbs {
                                if max_dtw(subtrajectory, rta.0) < spatial_deviation {
                                    local_hash_set.insert(ReferenceSubTrajectory(rta.0));
                                }
                                if max_dtw(subtrajectory, rtb.0) < spatial_deviation {
                                    local_hash_set.insert(ReferenceSubTrajectory(rtb.0));
                                }
                                if &rta.0.last() == &rtb.0.first() {
                                    unsafe {
                                        local_hash_set.insert(ReferenceSubTrajectory(
                                            std::slice::from_raw_parts(
                                                rta.0.as_ptr(),
                                                rta.0.len() + 1,
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
        trajectory_mrt_dict
    }
}
