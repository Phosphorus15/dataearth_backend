use actix_web::web::Data;
use std::sync::{Arc, Mutex};
use crate::database::{DatabaseAccess, UnifiedData};
use actix_web::{HttpRequest, Responder, HttpResponse, web, error, Error};
use actix_multipart::{Multipart, Field, MultipartError};
use futures::{Stream, Future};
use std::fs;
use futures::future::{Either, err};
use std::io::Write;

pub fn init_token(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest, data: actix_web::web::Json<UnifiedData>) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        let guard = database.lock().unwrap();
        if i.user_type == 1 && !guard.try_init() {
            guard.feed_init(data.0.clone());
            return HttpResponse::Ok().content_type("application/json").body("{\"result\": true}");
        }
    }
    HttpResponse::Ok().content_type("application/json").body("{\"result\": false}")
}

pub fn request_unified_data(database: Data<Arc<Mutex<DatabaseAccess>>>) -> impl Responder {
    HttpResponse::Ok().content_type("application/json")
        .body(serde_json::to_string(&database.lock().unwrap().load_init()).unwrap())
}

pub fn save_file(field: Field, file_path_string: &str, auth: bool) -> impl Future<Item=i64, Error=Error> {
    if !auth {
        return Either::A(err(error::ErrorForbidden(
            "Not authenticated !"
        )));
    }
    let file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
    };
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                // fs operations are blocking, we have to execute writes
                // on threadpool
                web::block(move || {
                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        MultipartError::Payload(error::PayloadError::Io(e))
                    })?;
                    acc += bytes.len() as i64;
                    Ok((file, acc))
                })
                    .map_err(|e: error::BlockingError<MultipartError>| {
                        match e {
                            error::BlockingError::Error(e) => e,
                            error::BlockingError::Canceled => MultipartError::Incomplete,
                        }
                    })
            })
            .map(|(_, acc)| acc)
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn upload_road_data(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest, multipart: Multipart) -> impl Future<Item=HttpResponse, Error=Error> {
    let info = crate::login::get_login(database.clone(), request);
    let mut auth = false;
    if let Some(i) = info {
        if i.user_type == 1 {
            auth = true;
        }
    }
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(move |field| save_file(field, "road_data.geojson", auth).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

pub fn upload_point_data(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest, multipart: Multipart) -> impl Future<Item=HttpResponse, Error=Error> {
    let info = crate::login::get_login(database.clone(), request);
    let mut auth = false;
    if let Some(i) = info {
        if i.user_type == 1 {
            auth = true;
        }
    }
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(move |field| save_file(field, "point_data.geojson", auth).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}
