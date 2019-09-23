use crate::database::{DatabaseAccess, PoliceStation, Position};
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::{Deserialize, Serialize};
use actix_web::{HttpRequest, Responder, HttpResponse};

#[derive(Deserialize)]
pub struct DeletePoliceStationInfo {
    id: String
}

#[derive(Deserialize, Serialize)]
pub struct Crew {
    id: String,
    name: String,
}

#[derive(Deserialize, Serialize)]
pub struct AddPoliceStationInfo {
    id: String,
    name: String,
    position: Position,
    crew: Vec<Crew>,
    drones: i32,
}

#[derive(Serialize)]
pub struct PoliceStations {
    inner: Vec<PoliceStation>
}

pub fn delete_police_station(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeletePoliceStationInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.lock().unwrap().delete_police_station(login.id.clone());
            return HttpResponse::Ok().content_type("application/json").body("{\"result\": \"success\"}");
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}

pub fn list_police_station(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(_i) = info {
        return HttpResponse::Ok().content_type("application/json").body(
            serde_json::to_string(&PoliceStations{
                inner: database.lock().unwrap().find_police_station()
            }).unwrap()
        );
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}

pub fn add_police_station(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<AddPoliceStationInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.lock().unwrap().add_police_station(
                PoliceStation {
                    id: login.id.clone(),
                    name: login.name.clone(),
                    position: login.position,
                    crew: login.crew.iter().map(|crew| crew.name.clone()).collect(),
                    drones: login.drones,
                }
            );
            return HttpResponse::Ok().content_type("application/json").body("{\"result\": \"success\"}");
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}
