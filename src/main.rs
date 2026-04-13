use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files::Files;
use actix_cors::Cors;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use base64::decode;

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

    // remove prefixo base64
    let base64_data = match data.split(",").nth(1) {
        Some(v) => v,
        None => {
            return HttpResponse::BadRequest().body("imagem inválida");
        }
    };

    let bytes = match decode(base64_data) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().body("erro ao decodificar imagem");
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
        return HttpResponse::InternalServerError().body("erro salvar arquivo");
    }

    println!("Imagem salva em: {}", filename);

    HttpResponse::Ok().json("ok")
}

// rota pra teste
async fn index() -> impl Responder {
    HttpResponse::Ok().body("API rodando 🚀")
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

            .route("/", web::get().to(index))
            .route("/registrar", web::post().to(registrar))

            // opcional: acessar arquivos
            .service(Files::new("/files", "/tmp").show_files_listing())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}