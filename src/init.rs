use actix_web::web::Data;
use std::sync::{Arc, Mutex};
use crate::database::{DatabaseAccess, UnifiedData};
use actix_web::{HttpRequest, Responder, HttpResponse};

pub fn init_token(database: Data<Arc<Mutex<DatabaseAccess>>>, request: HttpRequest, data: actix_web::web::Json<UnifiedData>) -> impl Responder {
    let info = crate::login::get_login(database.clone(), request);
    if let Some(i) = info {
        let guard = database.lock().unwrap();
        if i.user_type == 1 && ! guard.try_init() {
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
