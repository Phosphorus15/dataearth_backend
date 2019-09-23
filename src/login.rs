use actix_web::{ Responder, HttpMessage};
use actix_web::HttpRequest;
use actix_web::web::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::database::DatabaseAccess;
use actix_web::cookie::CookieBuilder;
use actix::Addr;
use crate::dispatcher::DispatcherService;
use crate::dispatch::{Workload, Coordinates};

#[derive(Deserialize)]
pub struct LoginInfo {
    name: String,
    passwd: String,
    user_type: i32,
}

#[derive(Serialize)]
pub struct LoginResult {
    result: &'static str
}

pub fn get_login(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> Option<crate::database::LoginInfo> {
    let cookie = request.cookie("sess");
    match cookie {
        None => {
            None
        }
        Some(token) => {
            database.lock().unwrap().find_login(token.value().to_string())
        }
    }
}

pub fn get_login_type(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        return HttpResponse::Ok().content_type("application/json").body(format!("{{\"type\": {} }}", i.user_type));
    }
    HttpResponse::Ok().content_type("application/json").body("{\"type\": -1}")
}

pub fn user_login(database: Data<Arc<Mutex<DatabaseAccess>>>, login: Json<LoginInfo>) -> impl Responder {
    let db = database.lock().unwrap();
    let user = db.find_user(login.name.clone());
    match user {
        None => {
            HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&LoginResult {
                result: "User not found !"
            }).unwrap())
        }
        Some(user) => {
            if user.passwd == login.passwd {
                if user.user_type == login.user_type {
                    let uuid = uuid::Uuid::new_v4().to_string();
                    db.add_login(crate::database::LoginInfo {
                        username: user.username.clone(),
                        user_type: login.user_type,
                        token: uuid.to_string(),
                    });
                    HttpResponse::Ok().content_type("application/json").cookie(CookieBuilder::new("sess", uuid).path("/").secure(false).finish()).body(serde_json::to_string(&LoginResult {
                        result: "success"
                    }).unwrap())
                } else {
                    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&LoginResult {
                        result: "User not found !"
                    }).unwrap())
                }
            } else {
                HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&LoginResult {
                    result: "Password is wrong !"
                }).unwrap())
            }
        }
    }
}

