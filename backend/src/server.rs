mod route_provider;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, Result, Responder, get, post, HttpResponse, http};
use actix_files;
use chrono::DateTime;
use serde::Deserialize;
use serde_json::json;
use std::{env, path::PathBuf};
use std::str;
use std::fs;
use log::info;
use log::debug;
use serde_json;

use trailscout_lib::algorithm::Algorithm;
use trailscout_lib::data::graph::{Graph, Sight};
use crate::route_provider::RouteProviderRes;

///Location of the application config file
const CONFIG_PATH :&str = "./config.json";

///Represents state containing the config and appstate
struct AppState {
    graph: Graph,
    config: Config,
}


///Deserialization of config file
#[derive(Deserialize, Debug, Clone)]
struct Config {
    ip: String,
    port: u16,
    log_level: String,
    graph_file_path : String,
    routing_algorithm: String,
}

///read config.json at CONFIG_PATH and return it
fn get_config() -> Config {

    //using println instead of logging because this runs before logger can be initialized
    println!("Trying to read config at {}", CONFIG_PATH);

    let data = fs::read_to_string(CONFIG_PATH).expect("Unable to read file");
    let config: Config = serde_json::from_str(&data).expect("Unable to parse");

    println!("Read config:\n{:#?}", &config);

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

///Responds to post request asking for sights
#[post("/sights")]
async fn post_sights(request:  web::Json<SightsRequest>, data: web::Data<AppState>) -> impl Responder {

    debug!("Got Sights Request for lat={}, lon={} and radius={}.",
        request.lat, request.lon, request.radius);

    let sights = data.graph.get_sights_in_area(
        request.lat, request.lon, request.radius).values().cloned().collect::<Vec<&Sight>>();

    let mut res = HttpResponse::Ok();
    res.json(json!(sights))
}



///Responds to post request asking for routing
#[post("/route")]
async fn post_route(request:  web::Json<route_provider::RouteProviderReq>, data: web::Data<AppState>) -> impl Responder {
    debug!("Received route request");

    let route_request = request.into_inner();

    //parse start and end from Iso 8601 (rfc3339)
    let start = DateTime::parse_from_rfc3339(&route_request.start)
        .expect("Timer Parse Error");
    let end = DateTime::parse_from_rfc3339(&route_request.end)
        .expect("Timer Parse Error");

    //convert km/h to m/s
    let speed_mps = route_request.walking_speed_kmh as f64 / 3.6;

    //get configured algorithm
    let algo = Algorithm::from_name(&data.config.routing_algorithm,
                                    &data.graph,
                                    DateTime::from(start),
                                    DateTime::from(end),
                                    speed_mps,
                                    route_request.area,
                                    route_request.user_prefs).unwrap();
    let route = algo.compute_route();

    debug!("Computed route with {}. Sending response...", &data.config.routing_algorithm);

    HttpResponse::Ok().json(RouteProviderRes {
        route,
    })
}



//server main
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config: Config = get_config();

    // Initialize logger
    env::set_var("RUST_LOG", &config.log_level);
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();


    debug!("Starting to parsed graph from: {}", &config.graph_file_path);
    let graph = Graph::parse_from_file(&config.graph_file_path).expect("Error parsing graph from file");
    debug!("Parsed graph from: {}", &config.graph_file_path);


    let data = web::Data::new(AppState {
        graph,
        config: config.clone(),
        //rw_lock_graph : Arc::new(RwLock::new(Graph::new())),
    });

    HttpServer::new(move|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .service(post_sights)
            .service(post_route)
            .app_data(data.clone())

    })
    .bind((config.ip, config.port))?
    .run()
    .await
}
