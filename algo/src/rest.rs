use crate::max_dtw::max_dtw;
use std::collections::{HashMap, HashSet};

pub type TrajectoryReference<'a> = &'a [TwoPrecisionFixedPointPoint];

impl From<(f32, f32)> for TwoPrecisionFixedPointPoint {
    fn from(value: (f32, f32)) -> TwoPrecisionFixedPointPoint {
        TwoPrecisionFixedPointPoint {
            lat: (value.0 * 100.0) as i32,
            lng: (value.1 * 100.0) as i32,
        }
    }
}
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TwoPrecisionFixedPointPoint {
    pub lat: i32,
    pub lng: i32,
}
impl TwoPrecisionFixedPointPoint {
    pub fn distance(&self, other: &TwoPrecisionFixedPointPoint) -> i32 {
        let dx = self.lat - other.lat;
        let dy = self.lng - other.lng;
        dx.pow(2) + dy.pow(2)
    }
}

pub fn mrt_search<'a>(
    trajectory: &'a TrajectoryReference,
    reference_set: HashSet<&'a TrajectoryReference>,
    spatial_deviation: i32,
    // I want the return type to be reference to polyline(inside hashset), but something about not knowing size at
    // compile time
) -> HashMap<(usize, usize), HashSet<&'a [TwoPrecisionFixedPointPoint]>> {
    //index (0,1): { MRTs }
    let mut trajectory_mrt_dict: HashMap<(usize, usize), HashSet<TrajectoryReference>> =
        HashMap::new();

    // Initialize lenght two subtrajectory MRTs
    for i in 0..trajectory.len() - 2 {
        let mut local_hash_set: HashSet<TrajectoryReference> = HashSet::new();
        for rt in reference_set {
            for j in 0..rt.len() - 2 {
                if max_dtw(&trajectory[i..i + 1], &rt[j..j + 1]) < spatial_deviation {
                    local_hash_set.insert(&rt[j..j + 1]);
                }
            }
        }
        trajectory_mrt_dict.insert((i, i + 1), local_hash_set);
    }

    // Attempt to increase to length 3 subtrajectories, greedily
    for length in 3..=trajectory.len() {
        for i in 0..trajectory.len() - length + 1 {
            let subtrajectory = &trajectory[i..i + length];
            let rtas = trajectory_mrt_dict.get(&(i, i + length - 1));
            let rtbs = trajectory_mrt_dict.get(&(i + length - 1, i + length));
            let mut local_hash_set: HashSet<TrajectoryReference> = HashSet::new();
            // match, length 3 subtrajectories into sub,last
            match (rtas, rtbs) {
                (Some(rtas), Some(rtbs)) => {
                    for rta in rtas {
                        for rtb in rtbs {
                            if max_dtw(subtrajectory, rta) < spatial_deviation {
                                local_hash_set.insert(rta);
                            }
                            if max_dtw(subtrajectory, rtb) < spatial_deviation {
                                local_hash_set.insert(rtb);
                            }
                            // Here I want to compare the reference, not the value
                            // We replace the concept of "id" in the paper with rusts reference
                            // system, since all is done in memory
                            if &rta.last() == &rtb.first() {
                                //let rta_and_rtb = rta.union(rtb);
                                let rta_and_rtb = &rta[..rta.len() + 1]; // I actually just want to make a new
                                                                         // reference from the start of rta to the end of rtb
                                                                         // not do any real operation
                                                                         // would be lovely if I could assert that
                                                                         // rta and rtb were next to each other in memory
                                local_hash_set.insert(rta_and_rtb);
                            }
                        }
                    }
                }
                _ => break,
            }
        }
    }
    trajectory_mrt_dict
}
