use actix_web::{web, App, HttpServer, Result,Responder, get, post};
use actix_files;
use serde::Deserialize;
use std::{path::PathBuf};
use std::str;

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
    HttpServer::new(|| {
        App::new()
            .service(get_sights)
            .service(post_route)
            .service(actix_files::Files::new("/static", "../gui/dist/").show_files_listing())
            .service(actix_files::Files::new("/assets", "../gui/dist/assets").show_files_listing())
            .default_service(web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}