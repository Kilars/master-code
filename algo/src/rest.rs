use crate::max_dtw::max_dtw;
use std::collections::{HashMap, HashSet};

impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Point {
        Point {
            lat: (value.0 * 100000.0) as i32,
            lng: (value.1 * 100000.0) as i32,
        }
    }
}
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Point {
    pub lat: i32,
    pub lng: i32,
}
impl Point {
    pub fn distance(&self, other: &Point) -> i32 {
        let dx = self.lat - other.lat;
        let dy = self.lng - other.lng;
        dx.pow(2) + dy.pow(2)
    }
}
#[derive(PartialEq, Eq, Hash)]
enum SubTrajectory<'a> {
    Trajectory(Vec<Point>),
    Reference(&'a [Point]),
}
#[derive(PartialEq, Eq, Hash)]
struct ReferenceSubTrajectory<'a>(&'a [Point]);

struct EncodedTrajectory<'a>(Vec<SubTrajectory<'a>>);
struct ReferenceSet(HashSet<Vec<Point>>);
struct SubTrajectoryReferenceMap(HashMap<(usize, usize), ReferenceSet>);
struct SubTrajectoryPureReferenceMap<'a>(
    HashMap<(usize, usize), HashSet<ReferenceSubTrajectory<'a>>>,
);

impl ReferenceSet {
    fn encode(
        &self,
        trajectory: Vec<Point>,
        spatial_deviation: i32,
    ) -> SubTrajectoryPureReferenceMap {
        let mut trajectory_mrt_dict: SubTrajectoryPureReferenceMap =
            SubTrajectoryPureReferenceMap(HashMap::new());

        for i in 0..trajectory.len() - 2 {
            let mut local_hash_set: HashSet<ReferenceSubTrajectory> = HashSet::new();
            for rt in self.0.iter() {
                for j in 0..rt.len() - 2 {
                    if max_dtw(&trajectory[i..i + 1], &rt[j..j + 1]) < spatial_deviation {
                        local_hash_set.insert(ReferenceSubTrajectory(&rt[j..j + 1]));
                    }
                }
            }
            trajectory_mrt_dict.0.insert((i, i + 1), local_hash_set);
        }

        for length in 3..=trajectory.len() {
            for i in 0..trajectory.len() - length + 1 {
                let subtrajectory = &trajectory[i..i + length];
                let rtas = trajectory_mrt_dict.0.get(&(i, i + length - 1));
                let rtbs = trajectory_mrt_dict.0.get(&(i + length - 1, i + length));
                let mut local_hash_set: HashSet<ReferenceSubTrajectory> = HashSet::new();
                // match, length 3 subtrajectories into sub,last
                match (rtas, rtbs) {
                    (Some(rtas), Some(rtbs)) => {
                        for rta in rtas {
                            for rtb in rtbs {
                                if max_dtw(subtrajectory, rta.0) < spatial_deviation {
                                    // I first thought to do this, but it didnt work because the
                                    // hashset cannot take ownership.

                                    //local_hash_set.insert(*rtb.0);

                                    //Make a new reference, to the same thing, reference owned by
                                    //hashset
                                    local_hash_set.insert(ReferenceSubTrajectory(rta.0));
                                }
                                if max_dtw(subtrajectory, rtb.0) < spatial_deviation {
                                    local_hash_set.insert(ReferenceSubTrajectory(rtb.0));
                                }
                                if &rta.0.last() == &rtb.0.first() {
                                    let rta_and_rtb = &rta.0[..rta.0.len() + 1]; // I actually just want to make a new
                                                                                 // reference from the start of rta to the end of rtb
                                                                                 // not do any real operation
                                                                                 // would be lovely if I could assert that
                                                                                 // rta and rtb were next to each other in memory
                                    local_hash_set.insert(ReferenceSubTrajectory(rta_and_rtb));
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
}
