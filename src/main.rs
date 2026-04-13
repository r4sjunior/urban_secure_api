use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt as _;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

async fn upload(mut payload: Multipart) -> impl Responder {
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        // nome único do arquivo
        let filename = format!("upload-{}.bin", Uuid::new_v4());
        let filepath = format!("./uploads/{}", filename);

        println!("💾 Salvando em: {}", filepath);

        let mut f = File::create(&filepath).unwrap();

        // salva os chunks
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f.write_all(&data).unwrap();
        }
    }

    HttpResponse::Ok().body("Upload salvo com sucesso 🚀")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 10000;

    // garante pasta uploads
    std::fs::create_dir_all("./uploads").unwrap();

    println!("🔥 Server rodando na porta {}", port);

    HttpServer::new(|| {
        App::new()
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024))
            .wrap(Cors::permissive())

            // upload
            .route("/upload", web::post().to(upload))

            // site (igual antes)
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}