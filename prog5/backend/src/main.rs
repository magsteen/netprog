use actix_cors::Cors;
use actix_web::{get, http, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer};
use std::path::Path;

mod controller;

const HTTP_SERVER_ADDRESS: &str = "localhost:8080";

// Listen for incomming http requests
// Deserialize to json
// Establish language used -> create container for compiling/executing this language
// Send main.rs file as request to container for compilation and execution.
// Recieve response from execution, or determine runtime error if taking to long.
// Send response json to client.

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting HTTP server: go to http://localhost:8080");
    let b = Path::new("./dockers/py3/main.py").is_file();
    println!("{}", b);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .expose_any_header()
            .max_age(20000);
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(controller::main_controller::index)
            .service(controller::comprun_controller::service())
    })
    .bind(("0.0.0.0", 8080))? //or use address constant if possible?
    .run()
    .await
}
