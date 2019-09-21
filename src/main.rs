use actix_web::{HttpServer, App, Responder, HttpMessage};
use actix_web::web::*;

mod database;
mod login;
mod user;
mod dispatch;
mod police_station;
mod operator_mark;
mod init;
mod dispatcher;

use actix_web_static_files;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::database::DatabaseAccess;
use sha2::{Sha256, Digest};
use crate::dispatcher::DispatcherService;
use actix::{System, SyncArbiter};
use crate::dispatch::{Dispatcher, parse_road_data, construct_topology, offline_bellman_ford};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn init_check(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        return HttpResponse::Ok().content_type("application/json").body(format!("{{\"result\": {} }}", database.lock().unwrap().try_init()));
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": false}")
}

fn main_page(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    if login::get_login(database, request).is_none() {
        HttpResponse::Found().header("Location", "/static/login.html").finish()
    } else {
        HttpResponse::Found().header("Location", "/static/mainframe.html").finish()
    }
}

pub fn fast_sha256(data: &str) -> String {
    let mut sha = Sha256::new();
    sha.input(data.as_bytes());
    hex::encode(sha.result())
}

fn main() {
    let sys = actix::System::new("actix-server");
    let database = database::DatabaseAccess::new(
        //include_str!("../database.auth")
        "postgres://postgres:12345@localhost:5432"
    );
    database.init();
    database.try_init();
    let roadmap = parse_road_data(&include_str!("../graph_test.geojson").to_string()).unwrap();
    let graph = construct_topology(&roadmap);
    let optimized = offline_bellman_ford(&graph);
    let dispatcher = Dispatcher::new(graph, optimized);
    let arc = Arc::new(Mutex::new(database));
    let service_arc = arc.clone();
    let service = SyncArbiter::start(1, move || DispatcherService::new(service_arc.clone(), dispatcher.clone()));

    let wrapped_db = Data::new(arc.clone());
    HttpServer::new(move || {
        let generated = generate();
        App::new()
            .register_data(Data::new(service.clone()))
            .register_data(wrapped_db.clone())
            .service(actix_web_static_files::ResourceFiles::new(
                "/static",
                generated,
            ))
            .route("/user/login", post().to(login::user_login))
            .route("/", get().to(main_page))
            .route("/user/delete", post().to(user::delete_user))
            .route("/user/logout", post().to(user::logout))
            .route("/user/type", post().to(login::get_login_type))
            .route("/init/check", post().to(init_check))
            .route("/init/ps", post().to(police_station::add_police_station))
            .route("/user/add", post().to(user::add_user))
            .route("/data/mark", post().to(operator_mark::add_mark))
            .route("/data/init", post().to(init::init_token))
            .route("/data/request", post().to(init::request_unified_data))
            .route("/data/get_mark", post().to(operator_mark::list_mark))
            .route("/mark/delete", post().to(operator_mark::delete_mark))
            .route("/data/get_ps", post().to(police_station::list_police_station))
            .route("/ps/delete", post().to(police_station::delete_police_station))
            .route("/data/mark/ping", post().to(operator_mark::update_mark))
    })
        .bind("127.0.0.1:80").unwrap()
        .start();
    sys.run();
}
