use actix_web::{web, App, HttpServer, Result,Responder, get, post};
use actix_files;
use serde::Deserialize;
use std::{path::PathBuf};
use std::str;
use std::fs;
use serde_json;

const CONFIG_PATH :&str = "src/config.json";

//Deserialization of config
#[derive(Deserialize)]
struct Config {
    ip: String,
    port: u16,
}

//read config at CONFIG_PATH and return it
fn get_config() -> Config {

    let data = fs::read_to_string(CONFIG_PATH).expect("Unable to read file");
    let config: Config = serde_json::from_str(&data).expect("Unable to parse");
    println!("Read config: IP {}, Port {}", config.ip, config.port);

    return config;
}

//struct to contain parameters from get_sights request
#[derive(Debug, Deserialize)]
pub struct SightsRequest {
   lat: u64,
   lon: u64,
   radius: u64,
}

#[derive(Debug, Deserialize)]
pub struct RouteRequest {
   node_list: Vec<String>,
}

//Default service: sends all not differently handled requests to angular index.html
async fn index() -> Result<actix_files::NamedFile> {
    let path: PathBuf = "../gui/dist/index.html".parse().unwrap();
    Ok(actix_files::NamedFile::open(path)?)
}

//Get Request to handle sights
#[get("/sights")]
async fn get_sights(request:  web::Query<SightsRequest>) -> Result<impl Responder> {
    //Placeholder => get sights by parameter and radius?
    let response =  format!("Placeholder Sights Request for lat={}, lon={} and radius={}.", request.lat, request.lon, request.radius);
    Ok(web::Json(response))
}

//Post Request to handle route request
#[post("/route")]
async fn post_route(request: web::Json<RouteRequest>) -> Result<impl Responder> {
    let response =  format!("Placeholder Route Request for lat={}", request.node_list.len());
    Ok(web::Json(response))
}


//server main
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let config: Config = get_config();

    HttpServer::new(|| {
        App::new()
            .service(get_sights)
            .service(post_route)
            .service(actix_files::Files::new("/static", "../gui/dist/").show_files_listing())
            .service(actix_files::Files::new("/assets", "../gui/dist/assets").show_files_listing())
            .default_service(web::get().to(index))
    })
    .bind((config.ip, config.port))?
    .run()
    .await
}