use crate::database::{DatabaseAccess, Position, OperatorMark};
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::{Deserialize, Serialize};
use actix_web::{HttpRequest, Responder, HttpResponse, HttpMessage};
use std::time::UNIX_EPOCH;

#[derive(Deserialize)]
pub struct DeleteMarkInfo {
    uid: i32
}

#[derive(Deserialize)]
pub struct AddMarkInfo {
    position: Position,
    level: i32,
    drone: bool,
    desc: String,
}

#[derive(Serialize)]
pub struct Marks {
    inner: Vec<OperatorMark>
}

pub fn delete_mark(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeleteMarkInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 2 {
            database.lock().unwrap().delete_mark(login.uid.clone());
            return HttpResponse::Ok().content_type("application/json").body("{\"result\": \"success\"}");
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}

pub fn list_mark(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        return HttpResponse::Ok().content_type("application/json").body(
            serde_json::to_string(&Marks { inner: database.lock().unwrap().find_mark() })
                .unwrap()
        );
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}

pub fn update_mark(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest, req: Json<Vec<i32>>) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        let marks = database.lock().unwrap().find_mark();
        let mut remove = req.iter()
            .filter(|v| !marks.iter().any(|p| p.uid as i32 == **v))
            .map(|v| OperatorMark {
                uid: *v as u128,
                level: 0,
                position: Position {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                desc: String::new(),
                height: 0.0,
                drone: false,
            }).collect::<Vec<_>>();
        let append = marks.into_iter()
            .filter(|p| !req.contains(&(p.uid as i32)));
        remove.extend(append);
        return HttpResponse::Ok().content_type("application/json").body(
            serde_json::to_string(&remove)
                .unwrap()
        );
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}

pub fn add_mark(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<AddMarkInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 0 {
            let uid = database.lock().unwrap().add_mark(OperatorMark {
                position: login.position,
                height: login.position.z,
                level: login.level,
                desc: login.desc.clone(),
                drone: login.drone,
                uid: std::time::SystemTime::now().duration_since(UNIX_EPOCH)
                    .unwrap().as_millis(),
            });
            return HttpResponse::Ok().content_type("application/json").body(
                format!(
                    "{{\"result\": \"success\", \"id\": {}}}"
                    , uid)
            );
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}
