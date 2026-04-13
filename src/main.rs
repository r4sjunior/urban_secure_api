use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt as _;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

async fn upload(mut payload: Multipart) -> impl Responder {
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return HttpResponse::BadRequest().body("Erro no upload"),
        };

        // 🔒 evita problema de borrow
        let field_name = {
            let cd = field.content_disposition();
            cd.get_name().unwrap_or("").to_string()
        };

        if field_name == "file" {
            let filename = format!("upload-{}.bin", Uuid::new_v4());

            let mut f = match File::create(&filename) {
                Ok(file) => file,
                Err(_) => return HttpResponse::InternalServerError().body("Erro ao criar arquivo"),
            };

            // leitura segura do stream
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(bytes) => bytes,
                    Err(_) => return HttpResponse::InternalServerError().body("Erro ao ler arquivo"),
                };

                if f.write_all(&data).is_err() {
                    return HttpResponse::InternalServerError().body("Erro ao salvar arquivo");
                }
            }

            return HttpResponse::Ok().body(format!("Arquivo salvo: {}", filename));
        }
    }

    HttpResponse::BadRequest().body("Nenhum arquivo enviado")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🔥 Server rodando na porta 8080");

    HttpServer::new(|| {
        App::new()
            // 🔥 limite de upload (50MB)
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024))
            .wrap(Cors::permissive())
            .route("/upload", web::post().to(upload))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}