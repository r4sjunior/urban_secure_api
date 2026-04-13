use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt as _;

async fn upload(mut payload: Multipart) -> impl Responder {
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        // ✅ FIX DO ERRO (sem and_then)
        let field_name = field
            .content_disposition()
            .get_name()
            .unwrap_or("file");

        println!("📦 Campo recebido: {}", field_name);

        while let Some(chunk) = field.next().await {
            let _data = chunk.unwrap();
        }
    }

    HttpResponse::Ok().body("Upload concluído 🚀")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 10000;

    println!("🔥 Server rodando na porta {}", port);

    HttpServer::new(|| {
        App::new()
            // limite 50MB
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024))
            .wrap(Cors::permissive())

            // rota upload
            .route("/upload", web::post().to(upload))

            // 🔥 SITE (igual antes)
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}