use std::sync::Arc;
use std::collections::VecDeque;
use std::borrow::{Borrow, BorrowMut};

pub struct Coordinates {
    x: f64,
    y: f64,
    h: f64,
}

pub struct RoadIntersection {
    id: usize,
    location: Coordinates,
    link_to: Vec<usize>,
}

pub struct GraphPoint {
    location: Coordinates,
    id: usize,
    paths: Vec<Path>, // Path to specific point
}

impl RoadIntersection {
    fn compute_distance(&self, other: &Self) -> f64 {
        ((self.location.x - other.location.x).powi(2)
            + (self.location.y - other.location.y).powi(2))
            .sqrt()
    }
}

type Path = Vec<(usize, usize)>;

type RoadGraph = Vec<RoadIntersection>;

fn offline_bellman_ford(graph: RoadGraph) {
    graph.iter().map(|pos| {
        let pos = pos;
        let mut queue: VecDeque<usize> = VecDeque::with_capacity(graph.len());
        let mut nearest: Vec<f64> = Vec::with_capacity(graph.len());
        let mut path: Vec<Path> = Vec::with_capacity(graph.len());
        for i in 0..graph.len() {
            if i == pos.id {
                nearest.push(0.0)
            } else {
                nearest.push(std::f64::MAX)
            }
        }
        for i in pos.link_to.iter() {
            let cur = &graph[*i];
            nearest[cur.id] = pos.compute_distance(cur);
            path[cur.id].push((pos.id, cur.id));
        }
        queue.push_back(pos.id);
        while !queue.is_empty() {
            let cur = &graph[queue.pop_front().unwrap()];

            for to in cur.link_to.iter() {
                let to = &graph[*to];
                if to.id == cur.id || to.id == pos.id {
                    continue
                } else {
                    let dis = pos.compute_distance(cur.borrow()) + cur.compute_distance(to.borrow());
                    if nearest[to.id] > dis {
                        // Update possible shortest path
                        nearest[to.id] = dis;
                        path[to.id].clear();
                        path[to.id].append(&mut path[cur.id].clone());
                        path[to.id].push((cur.id, to.id));
                        queue.push_back(to.id);
                    }
                }
            }
        }
    });
}
