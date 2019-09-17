use crate::database::DatabaseAccess;
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::Deserialize;
use actix_web::{HttpRequest, Responder, HttpResponse, HttpMessage};

#[derive(Deserialize)]
pub struct DeletePoliceStationInfo {
    id: String
}

pub fn delete_police_station(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeletePoliceStationInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.try_lock().unwrap().delete_police_station(login.id.clone());
            return HttpResponse::Ok().body("{result: \"success\"}")
        }
    }
    HttpResponse::Ok().body("{result: \"failed\"}")
}