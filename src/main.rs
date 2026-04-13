use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files::Files;
use actix_cors::Cors;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use base64::{engine::general_purpose, Engine as _};

#[derive(Deserialize)]
struct Arte {
    nome: String,
    descricao: String,
    latitude: f64,
    longitude: f64,
    imagem: String,
}

async fn registrar(info: web::Json<Arte>) -> impl Responder {
    let data = &info.imagem;

    let base64_data = match data.split(",").nth(1) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("imagem inválida"),
    };

    let bytes = match general_purpose::STANDARD.decode(base64_data) {
        Ok(b) => b,
        Err(e) => {
            println!("Erro decode: {:?}", e);
            return HttpResponse::BadRequest().body("erro base64");
        }
    };

    let filename = format!("/tmp/{}.png", Uuid::new_v4());

    let mut file = match File::create(&filename) {
        Ok(f) => f,
        Err(e) => {
            println!("Erro criar arquivo: {:?}", e);
            return HttpResponse::InternalServerError().body("erro criar arquivo");
        }
    };

    if let Err(e) = file.write_all(&bytes) {
        println!("Erro salvar arquivo: {:?}", e);
        return HttpResponse::InternalServerError().body("erro salvar");
    }

    println!("Imagem salva: {}", filename);

    HttpResponse::Ok().json("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let port = std::env::var("PORT")
        .unwrap_or("10000".to_string())
        .parse::<u16>()
        .unwrap();

    println!("🔥 Server rodando na porta {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024))

            // API
            .route("/registrar", web::post().to(registrar))

            // FRONTEND
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}