use actix_web::{web, App, HttpServer, Result,Responder, get, post, HttpResponse};
use actix_files;
use serde::Deserialize;
use std::{path::PathBuf};
use std::any::Any;
use std::str;
use std::fs;
use std::ops::Deref;
use actix_web::web::Data;
use serde_json;
use std::sync::{Arc, RwLock };
use algorithm_api::api::{compute_route_greedy, route_provider};
use data_api::api::graph::{Graph, ParseError, Sight};
use algorithm_api::api::route_provider::RouteProviderReq;

//TODO cleanup imports

const CONFIG_PATH :&str = "src/config.json";

// This struct represents state
struct AppState {

    //ToDo Actix Web uses Arc<> underneath the shared application data. Removes the need to wrap with an Arc<>?
    rw_lock_graph: Arc<RwLock<Graph>>
}


//Deserialization of config
#[derive(Deserialize)]
struct Config {
    ip: String,
    port: u16,
    graph_file_path : String,

}

//read config at CONFIG_PATH and return it
fn get_config() -> Config {

    let data = fs::read_to_string(CONFIG_PATH).expect("Unable to read file");
    let config: Config = serde_json::from_str(&data).expect("Unable to parse");
    println!("Read config: IP {}, Port {}", config.ip, config.port);

    return config;
}

//struct to contain parameters from get_sights request
//ToDo possibly refactor -> move somewhere else
#[derive(Deserialize)]
pub struct SightsRequest {
   lat: f64,
   lon: f64,
   radius: f64
}

//Default service: sends all not differently handled requests to angular index.html
async fn index() -> Result<actix_files::NamedFile> {
    let path: PathBuf = "../../gui/dist/index.html".parse().unwrap();
    Ok(actix_files::NamedFile::open(path)?)
}

//ToDo Test with graph
///Responds to post request asking for sights
#[post("/sights")]
async fn post_sights(request:  web::Json<SightsRequest>, data: web::Data<AppState>) -> Result<impl Responder> {

    println!("Placeholder Sights Request for lat={}, lon={} and radius={}.", request.lat, request.lon, request.radius);
    //let sights : Vec<Sight> = data.state_graph.get_sights_in_area(request.lat, request.lon, request.radius);

    //Ok(web::Json(sights))
    Ok(web::Json("sights"))
}



///Responds to post request asking for routing
#[post("/route")]
async fn post_route(request:  web::Json<RouteProviderReq>, data: web::Data<AppState>) -> Result<impl Responder> {

   // let request_inner = request.into_inner();
    println!("Placeholder Route Request");
    //ToDo: send Request to algo - get algo answer - send algo answer back
    let test = Arc::clone(&data.rw_lock_graph);
    let response =  compute_route_greedy(test, request.into_inner());

    Ok(web::Json("response"))
}



//server main
#[actix_web::main]
async fn main() -> std::io::Result<()> { 
    let config: Config = get_config();

    //TODO fix state - Arc RWlock?
    //let graph_result: Result<Graph, ParseError> = Graph::parse_from_file(&config.graph_file_path);
    //let currentGraph = graph_result.expect("Error parsing graph from file");
    //let graph_result: Graph = Graph::new();
    //let rw_lock_graph = Arc::new(RwLock::new(graph_result));


    //move
    HttpServer::new(|| {
        App::new()
            .service(post_sights)
            .service(post_route)
            .service(actix_files::Files::new("/static", "../../gui/dist/").show_files_listing())
            .service(actix_files::Files::new("/assets", "../../gui/dist/assets").show_files_listing())
            .default_service(web::get().to(index))
            .app_data(web::Data::new(AppState {
                //rw_lock_graph : Arc::new(RwLock::new(Graph::parse_from_file(&config.graph_file_path).expect("Error parsing graph from file"))),
                rw_lock_graph : Arc::new(RwLock::new(Graph::new())),
            }))

    })
    .bind((config.ip, config.port))?
    .run()
    .await
}