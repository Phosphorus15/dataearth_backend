use actix_web::{HttpServer, App, Responder};
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
use actix::Actor;
use crate::dispatch::{Dispatcher, parse_road_data, construct_topology, offline_bellman_ford};
use std::io::{BufReader, Read};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

fn init_check(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(_i) = info {
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
    let mut init = database.try_init();
    let file = std::fs::File::open("point_data.geojson");
    let dispatcher = if file.is_ok() && init {
        println!("Loading initial road map data...");
        let mut string = String::new();
        BufReader::new(file.unwrap()).read_to_string(&mut string).unwrap();
        let roadmap = parse_road_data(&string).unwrap();
        let graph = construct_topology(&roadmap);
        let optimized = offline_bellman_ford(&graph);
        Dispatcher::new(graph, optimized)
    } else {
        init = false;
        Dispatcher::new(vec![], vec![])
    };
    let arc = Arc::new(Mutex::new(database));
    let service_arc = arc.clone();
    let service = DispatcherService::new(service_arc.clone(), dispatcher.clone(), init).start();

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
            .route("/upload/road", post().to_async(init::upload_road_data))
            .route("/upload/point", post().to_async(init::upload_point_data))
            .route("/route", post().to(operator_mark::list_routes))
    })
        .bind("127.0.0.1:80").unwrap()
        .start();
    println!("Initialized : {}", init);
    println!("System is now running ...");
    sys.run().expect("Unable to start actix system");
}
