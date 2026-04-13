use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use actix_files::Files;
use futures_util::StreamExt;
use std::io::Write;
use uuid::Uuid;
use actix_cors::Cors;

async fn registrar(mut payload: Multipart) -> impl Responder {
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        let filename = format!("{}.bin", Uuid::new_v4());
        let filepath = format!("./uploads/{}", filename);

        let mut f = std::fs::File::create(filepath).unwrap();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f.write_all(&data).unwrap();
        }
    }

    HttpResponse::Ok().json("Upload realizado com sucesso")
}

async fn index() -> impl Responder {
    let html = std::fs::read_to_string("./static/index.html")
        .unwrap_or("Erro ao carregar HTML".to_string());

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("./uploads").ok();

    let port = 10000;

    println!("🔥 Server rodando na porta {}", port);

    HttpServer::new(|| {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024))
            .route("/", web::get().to(index))
            .route("/registrar", web::post().to(registrar))
            .service(Files::new("/uploads", "./uploads").show_files_listing())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}