use actix_cors::Cors;
use actix_web::{App, get, http, HttpResponse, HttpServer, post, Responder, Result, web};
use actix_files;
use chrono::DateTime;
use serde::Deserialize;
use serde_json::json;
use std::{env, path::PathBuf};
use std::any::Any;
use std::str;
use std::fs;
use log::info;
use log::debug;
use log::error;
use serde_json;

use trailscout_lib::algorithm::{Algorithm, AlgorithmError};
use trailscout_lib::data::graph::{Graph, Sight};
use trailscout_lib::server_utils::requests::{RouteProviderReq, RouteProviderRes, SightsRequest};
use trailscout_lib::server_utils::errors::{TailScoutError};

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
async fn post_route(request:  web::Json<RouteProviderReq>, data: web::Data<AppState>) -> Result<HttpResponse, TailScoutError> {
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
    let algo_result = Algorithm::from_name(&data.config.routing_algorithm,
                                    &data.graph,
                                    DateTime::from(start),
                                    DateTime::from(end),
                                    speed_mps,
                                    route_request.area,
                                    route_request.user_prefs);

    match algo_result {
        Ok(algo) => {

            let route = algo.compute_route();
            debug!("Computed route with {}. Sending response...", &data.config.routing_algorithm);

            Ok(HttpResponse::Ok().json(RouteProviderRes {
                route,
            }))
        }
        Err(error) => {
            error!("Error in post_route: {}",error);
            Err(TailScoutError::InternalError {message: format!("Error in post_route: {}",error)})

        }
    }



}

/// TODO Function kickoff a parse of a new pbf file to fmi graph
/// TODO Should also update the appstate if possible
async fn update_graph(){
    unimplemented!();
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
    let graph = Graph::parse_from_file(&config.graph_file_path).expect("Error parsing graph from file"); //parse error
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
