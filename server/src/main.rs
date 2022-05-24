use std::io;
use log::info;
use actix_web::http::StatusCode;
use actix_web::{get, post, guard, middleware, web, App, HttpResponse, HttpServer, Result,Responder};
use actix_files;
use std::path::PathBuf;


async fn index() -> Result<actix_files::NamedFile> {
    let path: PathBuf = "../gui/dist/index.html".parse().unwrap();
    Ok(actix_files::NamedFile::open(path)?)
}




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            //.route("/", web::get().to(index))
            .service(actix_files::Files::new("/static", "../gui/dist/").show_files_listing())
            .service(actix_files::Files::new("/assets", "../gui/dist/assets").show_files_listing())
            .default_service(web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}