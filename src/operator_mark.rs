use crate::database::DatabaseAccess;
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::Deserialize;
use actix_web::{HttpRequest, Responder, HttpResponse, HttpMessage};

#[derive(Deserialize)]
pub struct DeleteMarkInfo {
    uid: i32
}

pub fn delete_mark(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeleteMarkInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 2 {
            database.try_lock().unwrap().delete_mark(login.uid.clone());
            return HttpResponse::Ok().body("{result: \"success\"}")
        }
    }
    HttpResponse::Ok().body("{result: \"failed\"}")
}