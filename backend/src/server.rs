mod route_provider;

use actix_web::{web, App, HttpServer, Result, Responder, get, post, HttpResponse};
use actix_files;
use chrono::{DateTime, Utc, NaiveTime};
use serde::Deserialize;
use serde_json::json;
use std::{env, path::PathBuf};
use std::any::Any;
use std::borrow::Borrow;
use std::str;
use std::fs;
use std::ops::Deref;
use actix_web::web::Data;
use serde_json;

use trailscout_lib::algorithm::Algorithm;
use trailscout_lib::algorithm::greedy::GreedyAlgorithm;
use trailscout_lib::data::graph::{Graph, ParseError, Sight};
//TODO cleanup imports

const CONFIG_PATH :&str = "./config.json";

// This struct represents state
struct AppState {
    graph: Graph,
}


//Deserialization of config
#[derive(Deserialize, Debug)]
struct Config {
    ip: String,
    port: u16,
    log_level: String,
    graph_file_path : String,

}

//read config at CONFIG_PATH and return it
fn get_config() -> Config {

    let data = fs::read_to_string(CONFIG_PATH).expect("Unable to read file");
    let config: Config = serde_json::from_str(&data).expect("Unable to parse");
    log::info!("Read config:\n{:#?}", &config);

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


///Responds to post request asking for sights
#[post("/sights")]
async fn post_sights(request:  web::Json<SightsRequest>, data: web::Data<AppState>) -> impl Responder {

    println!("Placeholder Sights Request for lat={}, lon={} and radius={}.", request.lat, request.lon, request.radius);

    let sights = data.graph.get_sights_in_area(request.lat, request.lon, request.radius);

    //TODO does this serialize correctly according to interface definition?
    let mut res = HttpResponse::Ok();
    res.json(json!(sights))
}



///Responds to post request asking for routing
#[post("/route")]
async fn post_route(request:  web::Json<route_provider::RouteProviderReq>, data: web::Data<AppState>) -> impl Responder {

    println!("Placeholder Route Request");
    let route_request = request.into_inner();

    //parse start and end from Iso 8601 (rfc3339)
    let start = DateTime::parse_from_rfc3339(&route_request.start).expect("Timer Parse Error");
    let end = DateTime::parse_from_rfc3339(&route_request.end).expect("Timer Parse Error");

    //convert km/h to m/s
    let speed_mps = route_request.walking_speed_kmh as f64 / 3.6;

    let algo = GreedyAlgorithm::new(&data.graph, DateTime::from(start),
                                   DateTime::from(end), speed_mps, route_request.area, route_request.user_prefs);

    let route = algo.compute_route();
    let response = route_provider::RouteProviderRes{route};

    let mut res = HttpResponse::Ok();
    res.json(response)
}



//server main
#[actix_web::main]
async fn main() -> std::io::Result<()> { 
    let config: Config = get_config();

    // Initialize logger
    env::set_var("RUST_LOG", &config.log_level);
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    //TODO fix state - Arc RWlock?

    //TODO probably remove
    //let graph_result: Result<Graph, ParseError> = Graph::parse_from_file(&config.graph_file_path);
    //let currentGraph = graph_result.expect("Error parsing graph from file");
    //let graph_result: Graph = Graph::new();
    //let rw_lock_graph = Arc::new(RwLock::new(graph_result));


    let graph = Graph::parse_from_file(&config.graph_file_path).expect("Error parsing graph from file");
    log::info!("Parsed graph: {}", &config.graph_file_path);
    let data = web::Data::new(AppState {
        graph,
        //rw_lock_graph : Arc::new(RwLock::new(Graph::new())),
    });

    //move
    HttpServer::new(move|| {
        App::new()
            .service(post_sights)
            .service(post_route)
            .service(actix_files::Files::new("/static", "../../gui/dist/").show_files_listing())
            .service(actix_files::Files::new("/assets", "../../gui/dist/assets").show_files_listing())
            .default_service(web::get().to(index))
            .app_data(data.clone())

    })
    .bind((config.ip, config.port))?
    .run()
    .await
}