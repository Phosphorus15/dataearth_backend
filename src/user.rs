use crate::database::DatabaseAccess;
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::Deserialize;
use actix_web::{HttpRequest, Responder, HttpResponse};

#[derive(Deserialize)]
pub struct DeleteInfo {
    username: String
}

pub fn delete_user(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeleteInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.try_lock().unwrap().delete_user(login.username.clone());
            return HttpResponse::Ok().body("{result: \"success\"}")
        }
    }
    HttpResponse::Ok().body("{result: \"failed\"}")
}
