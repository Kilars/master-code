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
struct ReferenceSubTrajectory<'a> {
    slice: &'a [Point],
    used_index: usize,
}

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
    ) -> (EncodedTrajectory, f64) {
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
                    last_indexed_point += 1;
                    longest_mrt = Some(ReferenceSubTrajectory(next_mrt.0));
                }
                None => {
                    if let Some(longest_mrt) = longest_mrt.take() {
                        encoded_trajectory
                            .0
                            .push(SubTrajectory::Reference(longest_mrt.0));
                        last_indexed_point += 1;
                        first_point_not_indexed = last_indexed_point;
                        references += 1;
                    } else {
                        encoded_trajectory.0.push(SubTrajectory::Trajectory(
                            trajectory[first_point_not_indexed..=last_indexed_point].to_vec(),
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
        let point_size = 4.0;
        // 8 byte reference
        let reference_size = 8.0;

        let compression_ratio = (length as f64 * point_size)
            / ((direct_points as f64 * point_size) + (references as f64 * reference_size));

        (encoded_trajectory, compression_ratio)
    }

    fn get_mrt_set_len_two_st(
        &self,
        len_two_st: [Point; 2],
        spatial_deviation: f64,
    ) -> HashSet<ReferenceSubTrajectory> {
        let mut mrt_set: HashSet<ReferenceSubTrajectory> = HashSet::new();
        for rt in self.0.iter() {
            for j in 0..rt.len() - 1 {
                if max_dtw(&len_two_st.to_vec(), &rt[j..=j + 1]) < spatial_deviation {
                    mrt_set.insert(ReferenceSubTrajectory {
                        slice: &rt[j..],
                        used_index: j + 1,
                    });
                }
            }
        }
        mrt_set
    }
    fn expand_mrt(
        &self,
        st: &[Point],
        rt: ReferenceSubTrajectory,
        spatial_deviation: f64,
    ) -> HashSet<ReferenceSubTrajectory> {
        let mut mrt_set = HashSet::new();
        let rta = &rt.slice[..=rt.used_index];
        let rtb = &rt.slice[rt.used_index..=rt.used_index + 1];
        let rtab = &rt.slice[..=rt.used_index + 1];
        if max_dtw(st, rta) < spatial_deviation {
            mrt_set.insert(rt);
        }
        if max_dtw(st, rtb) < spatial_deviation {
            mrt_set.insert(ReferenceSubTrajectory {
                slice: rtb,
                used_index: rt.used_index + 1,
            });
        }
        if max_dtw(st, rtab) < spatial_deviation {
            mrt_set.insert(ReferenceSubTrajectory {
                slice: rt.slice,
                used_index: rt.used_index + 1,
            });
        }
        mrt_set
    }
    fn get_subtrajectory_references_revamped(
        &self,
        trajectory: &Vec<Point>,
        spatial_deviation: f64,
    ) -> SubTrajectoryReferenceIndex {
        let mut trajectory_mrt_dict: SubTrajectoryReferenceIndex =
            SubTrajectoryReferenceIndex(HashMap::new());

        //calc base set of st len 2
        let mut global_start = 0;
        let mut global_end = 1;
        let mut trajectory_mrt_dict = self.get_mrt_set_len_two_st(
            [trajectory[global_start], trajectory[global_end]],
            spatial_deviation,
        );
        // attempt expand for this st
        let mut mrt_set: HashSet<ReferenceSubTrajectory> = HashSet::new();

        // for each reference try to expand for expanded st
        for rt in trajectory_mrt_dict {
            //expand loop scope
            let mut can_expand = true;
            let mut local_start = global_start;
            let mut local_end = global_end;
            while can_expand {
                let mut rt_set_local = HashSet::new();
                rt_set_local.insert(rt);
                let mut expandable_rts = rt_set_local
                    .iter()
                    .filter(|rt| rt.used_index + 1 < rt.slice.len())
                    .collect::<Vec<_>>();

                let mut st_local = &trajectory[local_start..=local_end];

                for expandable_rt in expandable_rts {
                    let expanded_set = self.expand_mrt(st_local, *expandable_rt, spatial_deviation);
                    match expanded_set.is_empty() {
                        true => {
                            rt_set_local = expanded_set;
                            st_local = &trajectory[global_start..=global_end];
                            global_end += 1;
                        }
                        false => {}
                    }
                }
            }
        }
        //continue working with the base set one by one
        todo!()
    }
    fn get_subtrajectory_references(
        &self,
        trajectory: &Vec<Point>,
        spatial_deviation: f64,
    ) -> SubTrajectoryReferenceIndex {
        let mut trajectory_mrt_dict: SubTrajectoryReferenceIndex =
            SubTrajectoryReferenceIndex(HashMap::new());

        let mut first_point_not_indexed = 0;
        let mut last_indexed_point = 1;

        while first_point_not_indexed < trajectory.len() - 1 {
            let mut local_hash_set: HashSet<ReferenceSubTrajectory> = HashSet::new();
            let subtrajectory = &trajectory[first_point_not_indexed..=last_indexed_point + 1];
            let rtas = trajectory_mrt_dict
                .0
                .get(&(first_point_not_indexed, last_indexed_point));

            let rtbs = trajectory_mrt_dict
                .0
                .get(&(last_indexed_point, last_indexed_point + 1));

            match (rtas, rtbs) {
                (Some(rtas), Some(rtbs)) => {
                    let begin = std::time::Instant::now();
                    for rta in rtas {
                        if max_dtw(subtrajectory, rta.0, &mut dtw_lookup) < spatial_deviation {
                            local_hash_set.insert(ReferenceSubTrajectory(rta.0));
                        }
                    }
                    for rtb in rtbs {
                        if max_dtw(subtrajectory, rtb.0, &mut dtw_lookup) < spatial_deviation {
                            local_hash_set.insert(ReferenceSubTrajectory(rtb.0));
                        }
                    }
                    for rta in rtas {
                        for rtb in rtbs {
                            if &rta.0.last() == &rtb.0.first() {
                                unsafe {
                                    local_hash_set.insert(ReferenceSubTrajectory(
                                        std::slice::from_raw_parts(rta.0.as_ptr(), rta.0.len() + 1),
                                    ));
                                }
                            }
                        }
                    }
                    if local_hash_set.len() > 0 {
                        trajectory_mrt_dict.0.insert(
                            (first_point_not_indexed, last_indexed_point + 1),
                            local_hash_set,
                        );
                        last_indexed_point += 1;
                    } else {
                        last_indexed_point += 1;
                        first_point_not_indexed = last_indexed_point;
                    }
                    println!("duration {:.2?}", begin.elapsed());
                }
                _ => {
                    last_indexed_point += 1;
                    first_point_not_indexed = last_indexed_point;
                }
            }
        }
        trajectory_mrt_dict
    }
}
