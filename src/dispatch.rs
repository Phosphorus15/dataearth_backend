use std::sync::{Arc, Mutex};
use binary_heap_plus::BinaryHeap;
use json::JsonValue;
use std::sync::atomic::AtomicUsize;
use crate::database::Position;

#[derive(Copy, Clone, Debug)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub h: f64,
}

impl From<Position> for Coordinates {
    fn from(p: Position) -> Self {
        Self {
            x: p.x,
            y: p.y,
            h: p.z,
        }
    }
}

type Edge = (usize, usize, f64);

#[derive(Debug)]
pub struct RoadIntersection {
    id: usize,
    location: Coordinates,
    link_to: Vec<usize>,
}

#[derive(Debug)]
pub struct RawPoint {
    r1: isize,
    r2: isize,
    location: Coordinates,
}

impl RoadIntersection {
    fn compute_distance(&self, other: &Self) -> f64 {
        self.location.compute_distance(&other.location)
    }
}

impl Coordinates {
    fn compute_distance(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2)
            + (self.y - other.y).powi(2))
            .sqrt()
    }
}

type Path = Vec<(usize, usize)>;

type RoadGraph = Vec<RoadIntersection>;

pub fn construct_topology(points: &Vec<RawPoint>) -> RoadGraph {
    let mut bound = points.iter().zip(0..points.len()).map(|(p, id)| RoadIntersection {
        location: p.location,
        id,
        link_to: vec![],
    }).collect::<Vec<_>>();
    for pos in 0..bound.len() {
        let info = &points[pos];
        let pos = &mut bound[pos];
        let vec1 = points.iter().zip(0..points.len())
            .filter(|(p, id)| *id != pos.id && (p.r1 == info.r1 || p.r2 == info.r1) && !pos.link_to.contains(id))
            .map(|(p, id)| (p.location.compute_distance(&pos.location), id)).collect::<Vec<_>>();
        for p in vec1.iter() {
            pos.link_to.push(p.1);
        }
        if info.r2 >= 0 {
            // try-connect policy - connect two more times
            let vec2 = points.iter().zip(0..points.len())
                .filter(|(p, id)| *id != pos.id && (p.r1 == info.r2 || p.r2 == info.r1 || p.r2 == info.r2) && !pos.link_to.contains(id))
                .map(|(p, id)| (p.location.compute_distance(&pos.location), id)).collect::<Vec<_>>();
            for p in vec2.iter() {
                pos.link_to.push(p.1);
            }
        }
    }
    bound
}

pub fn parse_road_data(geojson: &String) -> Result<Vec<RawPoint>, ()> {
    let object = json::parse(&geojson[..]).expect("failed to parse json");
    if let JsonValue::Object(object) = object {
        if let Some(JsonValue::Array(features)) = object.get("features") {
            return Ok(features.iter().filter_map(|v| {
                if let JsonValue::Object(feature) = v {
                    if let (Some(JsonValue::Object(property))
                        , Some(JsonValue::Object(geometry))) = (feature.get("properties"), feature.get("geometry")) {
                        let r1 = property.get("road1").unwrap().as_isize().unwrap();
                        let r2 = property.get("road2").unwrap().as_isize().unwrap();
                        if let Some(JsonValue::Array(coords)) = geometry.get("coordinates") {
                            let x = coords[0].as_f64().unwrap();
                            let y = coords[1].as_f64().unwrap();
                            return Some(RawPoint {
                                r1,
                                r2,
                                location: Coordinates {
                                    x,
                                    y,
                                    h: 0.0,
                                },
                            });
                        }
                    }
                }
                None
            }).collect());
        }
    }
    Err(())
}

#[test]
fn test_road_parse() {
    let roadmap = parse_road_data(&include_str!("../graph_test.geojson").to_string()).unwrap();
    let graph = construct_topology(&roadmap);
    let optimized = offline_bellman_ford(&graph);
    // ensure that all data are properly mapped
    assert!(optimized.iter().map(|v| v.iter().filter(|v1| v1.is_empty()).count()).sum::<usize>() <= optimized.len());
}

pub fn offline_bellman_ford(graph: &RoadGraph) -> Vec<Vec<Path>> {
    graph.iter().map(|pos| {
        let mut queue = BinaryHeap::with_capacity_by(graph.len(), |u: &Edge, v: &Edge| {
            v.2.partial_cmp(&u.2).unwrap()
        });
        let mut visited: Vec<bool> = Vec::with_capacity(graph.len());
        let mut nearest: Vec<f64> = Vec::with_capacity(graph.len());
        let mut path: Vec<Path> = Vec::with_capacity(graph.len());
        for i in 0..graph.len() {
            visited.push(false);
            path.push(vec![]);
            if i == pos.id {
                nearest.push(0.0)
            } else {
                nearest.push(std::f64::MAX)
            }
        }
        visited[pos.id] = true;
        for i in pos.link_to.iter() {
            let cur = &graph[*i];
            nearest[cur.id] = pos.compute_distance(cur);
            path[cur.id].push((pos.id, cur.id));
            queue.push((pos.id, *i, nearest[cur.id]));
        }
        while !queue.is_empty() {
            let edge = &queue.pop().unwrap();
            let cur = &graph[edge.1];
            if visited[cur.id] {
                continue;
            } else {
                visited[cur.id] = true
            }
            let dis = nearest[edge.0] + edge.2;
            if nearest[edge.1] >= dis {
                nearest[edge.1] = dis;
                path[edge.1] = path[edge.0].clone();
                path[edge.1].push((edge.0, edge.1));
                for sub in cur.link_to.iter() {
                    if *sub != edge.0 {
                        queue.push((cur.id, *sub, cur.compute_distance(&graph[*sub])));
                    }
                }
            }
        }
        path
    }).collect()
}

#[derive(Clone)]
pub struct Drone {
    pub power: usize,
    pub location: Coordinates,
    pub uid: String,
}

#[derive(Clone)]
pub struct Workload {
    pub is_remove: bool,
    pub id: usize,
    pub severity: usize,
    pub consumption: usize,
    pub location: Coordinates,
    pub assign_id: usize,
    pub drone: bool,
}

impl Workload {
    pub fn delete(assign: usize) -> Self {
        Self {
            is_remove: true,
            id: assign,
            severity: 0,
            consumption: 0,
            location: Coordinates {
                x: 0.0,
                y: 0.0,
                h: 0.0,
            },
            assign_id: assign,
            drone: false,
        }
    }
}

#[derive(Clone)]
pub struct Dispatch {
    pub id: usize,
    pub power: usize,
    pub severity: usize,
    pub location: Coordinates,
    pub assign: usize,
    pub source: String,
    pub to_id: usize,
}

#[derive(Clone)]
pub struct Mission {
    pub id: usize,
    pub power: usize,
    pub severity: usize,
    pub from: Coordinates,
    pub to: Coordinates,
    pub path_given: Vec<(f64, f64)>,
    pub predecessor: usize,
    pub source: String,
}

pub struct Dispatcher(RoadGraph, Vec<Vec<Path>>);

const DISPATCH_FACTOR: f64 = 3f64;

impl Dispatcher {
    // This should constantly be locked by a mutex
    pub fn new(graph: RoadGraph, paths: Vec<Vec<Path>>) -> Arc<Mutex<Self>> {
        Arc::new(
            Mutex::new(
                Self(graph, paths)
            )
        )
    }

    // heuristic function to assess witch dispatch policy to use
    fn assess_dispatch(dis1: f64, dis2: f64, sev: i32) -> bool {
        let sev = sev as f64;
        if dis2 <= dis1 {
            false
        } else {
            // slide the edge
            return ((dis2 - dis1) / dis1 * (sev * DISPATCH_FACTOR / 3f64 + 1f64) / DISPATCH_FACTOR).log2() > 0f64;
        }
    }

    fn next_sat<'x>(workload: &Workload, ongoing: &'x mut Vec<Dispatch>, resources: &'x mut Vec<Drone>) -> (usize, Option<Result<&'x mut Dispatch, &'x mut Drone>>) {
        let dispatch = ongoing.iter_mut().filter(|v| v.severity < workload.severity && v.power > 0)
            .map(|v| (v.location.compute_distance(&workload.location), v))
            .min_by(|v1, v2| v1.0.partial_cmp(&v2.0).unwrap());
        let drone = resources.iter_mut().filter(|v| v.power > 0)
            .map(|v| (v.location.compute_distance(&workload.location), v))
            .min_by(|v1, v2| v1.0.partial_cmp(&v2.0).unwrap());
        match (dispatch, drone) {
            (Some(v), None) => (v.1.power, Some(Ok(v.1))),
            (None, Some(v)) => (v.1.power, Some(Err(v.1))),
            (Some(v1), Some(v2)) =>
                if Self::assess_dispatch(v1.0, v2.0, (workload.severity - v1.1.severity) as i32) {
                    (v1.1.power, Some(Ok(v1.1)))
                } else {
                    (v2.1.power, Some(Err(v2.1)))
                }
            (None, None) =>
                (0, None)
        }
    }

    fn generate_route(&self, from: Coordinates, to: Coordinates) -> Vec<(f64, f64)> {
        let start = self.0.iter().map(|v| (from.compute_distance(&v.location), v))
            .min_by(|v1, v2| v1.0.partial_cmp(&v2.0).unwrap())
            .unwrap();
        let end = self.0.iter().map(|v| (to.compute_distance(&v.location), v))
            .min_by(|v1, v2| v1.0.partial_cmp(&v2.0).unwrap())
            .unwrap();
        if from.compute_distance(&to) <= start.0 + end.0 {
            return vec![(from.x, from.y), (to.x, to.y)];
        } else {
            let mut route = vec![(from.x, from.y)];
            route.extend(self.1[start.1.id][end.1.id].iter().map(|(p1, _)| {
                let p1 = &self.0[*p1];
                (p1.location.x, p1.location.y)
            }));
            route.append(&mut vec![(end.1.location.x, end.1.location.y), (to.x, to.y)]);
            route
        }
    }

    pub fn online_dispatch_round(&self, mut workload: Workload, ongoing: &mut Vec<Dispatch>, resources: &mut Vec<Drone>, global_id: &AtomicUsize) -> (Vec<Mission>, Workload) {
        let mut solution = Self::next_sat(&workload, ongoing, resources);
        let mut missions = vec![];
        while workload.consumption > 0 && solution.0 > 0 {
            if let Some(sol_to) = solution.1 {
                match sol_to {
                    Ok(sol) => {
                        let power = workload.consumption.min(sol.power);
                        missions.push(Mission {
                            id: global_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                            power,
                            severity: workload.severity,
                            from: sol.location,
                            to: workload.location,
                            path_given: self.generate_route(sol.location, workload.location),
                            predecessor: sol.id,
                            source: sol.source.clone(),
                        });
                        workload.consumption -= power;
                        sol.power -= power;
                    }
                    Err(sol) => {
                        let power = workload.consumption.min(sol.power);
                        missions.push(Mission {
                            id: global_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                            power,
                            severity: workload.severity,
                            from: sol.location,
                            to: workload.location,
                            path_given: self.generate_route(sol.location, workload.location),
                            predecessor: 0,
                            source: sol.uid.clone(),
                        });
                        workload.consumption -= power;
                        sol.power -= power;
                    }
                }
            }
            solution = Self::next_sat(&workload, ongoing, resources);
        }
        (missions, workload)
    }
}