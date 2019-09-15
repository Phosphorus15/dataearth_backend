use actix_web::{HttpServer, App, Responder, HttpMessage};
use actix_web::web::*;

mod database;
mod login;
mod user;

use actix_web_static_files;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::database::{User, DatabaseAccess};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn main_page(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    if login::get_login(database, request).is_none() {
        HttpResponse::Found().header("Location", "/static/login.html").finish()
    } else {
        HttpResponse::Ok().body("success")
    }
}

fn main() {
    let database = database::DatabaseAccess::default();
    database.init();
    let wrapped_db = Data::new(Arc::new(Mutex::new(database)));
    HttpServer::new(move || {
        let generated = generate();
        App::new()
            .register_data(wrapped_db.clone())
            .service(actix_web_static_files::ResourceFiles::new(
                "/static",
                generated,
            ))
            .route("/user/login", post().to(login::user_login))
            .route("/", get().to(main_page))
            .route("/user/delete", post().to(user::delete_user))
    })
        .bind("127.0.0.1:80").unwrap()
        .run().unwrap();
}
