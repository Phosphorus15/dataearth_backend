use crate::database::{DatabaseAccess, User};
use std::sync::{Mutex, Arc};
use actix_web::web::{Data, Json};
use serde::Deserialize;
use actix_web::{HttpRequest, Responder, HttpResponse, HttpMessage};

#[derive(Deserialize)]
pub struct DeleteUserInfo {
    username: String
}

#[derive(Deserialize)]
pub struct AddUserInfo {
    username: String,
    usertype: i32,
    password: String,
}

pub fn delete_user(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<DeleteUserInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.lock().unwrap().delete_user(login.username.clone());
            return HttpResponse::Ok().content_type("application/json").body("{result: \"success\"}");
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{result: \"failed\"}")
}

pub fn add_user(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<AddUserInfo>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        if i.user_type == 1 {
            database.lock().unwrap().add_user(User {
                username: login.username.clone(),
                user_type: login.usertype,
                passwd: login.password.clone(),
            });
            return HttpResponse::Ok().content_type("application/json").body("{\"result\": \"success\"}");
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
}

pub fn logout(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    let cookie = request.cookie("sess");
    match cookie {
        None => {
            HttpResponse::Ok().content_type("application/json").body("{\"result\": \"failed\"}")
        }
        Some(token) => {
            database.lock().unwrap().logout(token.value().to_string());
            return HttpResponse::Ok().content_type("application/json").body("{\"result\": \"success\"}");
        }
    }
}
