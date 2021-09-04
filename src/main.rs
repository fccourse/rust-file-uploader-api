use actix_cors::Cors;
use actix_multipart::Multipart;
use std::io::Write;

use actix_web::{App, HttpResponse, HttpServer, middleware, web, Error};
use clap::load_yaml;
use futures::{StreamExt, TryStreamExt};


async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!("./uploaded-images/{}", sanitize_filename::sanitize(&filename));

        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    Ok(HttpResponse::Ok().into())
}

#[actix_web::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all("./uploaded-images").unwrap();
    
    // Clap CLI    
    let yaml = load_yaml!("../cli.yaml");
    let command_line = clap::App::from_yaml(yaml).get_matches();
    let host = command_line.value_of("host").unwrap_or("127.0.0.1");
    let port = command_line.value_of("port").unwrap_or("9000");
   
    println!("File uploader server running on {}:{}", host, port);

    let _ = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                //.allowed_origin("*")
                //.allowed_methods(vec!["GET", "POST"])
                ,)
            .service(
                actix_web::web::resource("/")
                .route(actix_web::web::post().to(save_file)),
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await;

    Ok(())
}



