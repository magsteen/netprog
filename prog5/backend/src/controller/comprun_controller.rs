use std::fmt::Result;

use actix_web::{post, services, web, Handler, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Program {
    lang: String,
    source: String,
}

#[derive(Debug, Serialize)]
struct Response {
    result: String,
}

pub fn service() -> Scope {
    web::scope("/comprun").service(comprun)
}

#[post("")]
pub async fn comprun(program: web::Json<Program>) -> impl Responder {
    let result = format!(
        "You called comprun post (controller) with data: {}",
        program.lang
    );

    //HttpResponse::Ok().json(Response { result });
    web::Json(Response { result })
}
