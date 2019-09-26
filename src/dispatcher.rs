use crate::dispatch::*;
use actix::{Actor, Handler, Message};
use crate::database::{DatabaseAccess, DispatchedRoutes};
use std::sync::{Mutex, Arc};
use std::sync::atomic::AtomicUsize;
use actix::prelude::*;
use std::time::UNIX_EPOCH;

pub struct DispatcherService(Arc<Mutex<DatabaseAccess>>, Arc<Mutex<Dispatcher>>, Vec<Drone>, Vec<Dispatch>, AtomicUsize, bool);

impl DispatcherService {
    pub fn new(db: Arc<Mutex<DatabaseAccess>>, dispatcher: Arc<Mutex<Dispatcher>>, available: bool) -> Self {
        let drone = if available {
            db.lock().unwrap().find_police_station().iter().map(|ps| Drone {
                power: ps.crew.len(),
                location: Coordinates::from(ps.position),
                uid: ps.id.clone(),
            }).collect()
        } else {
            vec![]
        };
        DispatcherService(db, dispatcher, drone, vec![], // use millisecond-timestamp for id marking
                          AtomicUsize::new(std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as usize)
                          , available)
    }
}

impl Message for Workload {
    type Result = Result<(), ()>;
}

impl Actor for DispatcherService {
    type Context = Context<Self>;
}

impl Handler<Workload> for DispatcherService {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: Workload, ctx: &mut Self::Context) -> Self::Result {
        if !self.5 {
            return Err(());
        }
        let dispatcher = self.1.lock().unwrap();

        if msg.is_remove {
            let vec = &mut self.2;
            let database = self.0.lock().unwrap(); // lock for now
            self.3 = self.3.iter().cloned().filter_map(|v| {
                if v.assign == msg.assign_id {
                    for i in vec.iter_mut() {
                        if v.source == i.uid {
                            i.power += v.power // assign dispatched power back
                        }
                        database.remove_routes(v.to_id).unwrap();
                    }
                    None
                } else {
                    Some(v)
                }
            }).collect();
            return Ok(());
        }
        let dispatched =
            dispatcher.online_dispatch_round(msg.clone(), &mut self.3, &mut self.2, &self.4);
        if dispatched.1.consumption < msg.consumption { // or else there's no need to lock the database
            let database = self.0.lock().unwrap(); // lock for now
            for mission in dispatched.0.iter() {
                database.add_route(DispatchedRoutes {
                    route: mission.path_given.clone(),
                    belong: msg.id,
                });
                self.3.push(Dispatch {
                    id: self.4.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                    power: mission.power,
                    severity: mission.severity,
                    location: mission.to,
                    source: mission.source.clone(),
                    assign: msg.assign_id,
                    to_id: msg.id,
                })
            }
            drop(database); // drop the reference for now
        }
        if dispatched.1.consumption > 0 {
            ctx.address().do_send(dispatched.1);
        }
        Ok(())
    }
}
