use actix_web::{post, web, Responder, Scope};
use core::panic;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    process::{Command, Output},
};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Program {
    source: String,
}

#[derive(Debug, Serialize)]
struct Response {
    result: String,
}

const FOLDER: &str = "./dockers/py3";

pub fn service() -> Scope {
    web::scope("/comprun").service(comprun)
}

#[post("")]
pub async fn comprun(program: web::Json<Program>) -> impl Responder {
    let mut file = File::create("./dockers/py3/main.py").unwrap();
    write!(file, "{}", program.source).unwrap();

    let image_name = Uuid::new_v4().to_simple();

    create_image(FOLDER, image_name.to_string().as_str());

    let output = run_image(image_name.to_string().as_str());

    let out: Vec<u8>;
    if output.status.success() {
        out = output.stdout
    } else {
        out = output.stderr
    }

    let result = match std::str::from_utf8(&out) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    //Recieve program output as result and return that
    web::Json(Response { result })
}

fn create_image(folder: &str, image_name: &str) -> Output {
    println!("Creating image in folder: {}", folder);
    return Command::new("docker")
        .args(["image", "build", folder, "-t", image_name])
        .output()
        .unwrap();
}

fn run_image(image_name: &str) -> Output {
    println!("Running image");
    return Command::new("docker")
        .args(["run", "--rm", image_name])
        .output()
        .unwrap();
}
