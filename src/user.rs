use crate::database::DatabaseAccess;
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::Deserialize;
use actix_web::{HttpRequest, Responder, HttpResponse, HttpMessage};

#[derive(Deserialize)]
pub struct DeleteUserInfo {
    username: String
}

pub fn delete_user(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeleteUserInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.try_lock().unwrap().delete_user(login.username.clone());
            return HttpResponse::Ok().body("{result: \"success\"}")
        }
    }
    HttpResponse::Ok().body("{result: \"failed\"}")
}

pub fn logout(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeleteUserInfo>, request: HttpRequest) -> impl Responder {
    let cookie = request.cookie("sess");
    match cookie {
        None => {
            HttpResponse::Ok().body("{result: \"failed\"}")
        }
        Some(token) => {
            database.try_lock().unwrap().logout(token.value().to_string());
            return HttpResponse::Ok().body("{result: \"success\"}")
        }
    }

}
