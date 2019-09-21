use crate::dispatch::*;
use actix::{Actor, SyncContext, Handler, Message};
use crate::database::DatabaseAccess;
use std::sync::{Mutex, Arc};
use std::sync::atomic::AtomicUsize;

pub struct DispatcherService(Arc<Mutex<DatabaseAccess>>, Arc<Mutex<Dispatcher>>, Vec<Drone>, Vec<Dispatch>, AtomicUsize);

impl DispatcherService {
    pub fn new(db: Arc<Mutex<DatabaseAccess>>, dispatcher: Arc<Mutex<Dispatcher>>) -> Self {
        DispatcherService(db, dispatcher, vec![], vec![], AtomicUsize::new(0))
    }
}

impl Message for Workload {
    type Result = Result<(), ()>;
}

impl Actor for DispatcherService {
    type Context = SyncContext<Self>;
}

impl Handler<Workload> for DispatcherService {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: Workload, ctx: &mut Self::Context) -> Self::Result {
        let dispatcher = self.1.lock().unwrap();
        let dispatched =
            dispatcher.online_dispatch_round(msg.clone(), &self.3, &self.2, &self.4);
        Ok(())
    }
}
