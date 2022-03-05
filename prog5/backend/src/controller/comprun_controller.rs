use core::panic;
use std::{
    fmt::Result,
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Output},
};
use uuid::Uuid;

use actix_web::{post, services, web, Handler, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};

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

    let result = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    //HttpResponse::Ok().json(Response { result });

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

// fn create_entrypoint(timelimit: i32) {
//     let folder = "../dockers/py3";
//     let run_cmd = "python3";
//     let file = "main.py";

//     let execution_cmd = format!(
//         "timeout --signal=SIGTERM {}s {} {}\n",
//         timelimit, run_cmd, file
//     );

//     let file_content = "#!/usr/bin/env bash\n {}exit$?\n";
// }
