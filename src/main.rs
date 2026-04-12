use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use validator::Validate;
use log::info;

use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

#[derive(Debug, Deserialize, Validate)]
struct Entrada {
    #[validate(length(min = 3, max = 100))]
    nome: String,

    #[validate(length(min = 3, max = 100))]
    autor: String,

    #[validate(length(min = 5, max = 500))]
    descricao: String,

    imagem: String, // base64 direto

    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
struct Resposta {
    status: String,
}

// 🔥 SALVAR
async fn registrar(dados: web::Json<Entrada>) -> impl Responder {
    if let Err(e) = dados.validate() {
        return HttpResponse::BadRequest().json(format!("Erro: {}", e));
    }

    if dados.latitude.abs() > 90.0 || dados.longitude.abs() > 180.0 {
        return HttpResponse::BadRequest().json("Coordenadas inválidas");
    }

    info!("Novo registro recebido");

    let metadata = gerar_metadata(
        &dados.nome,
        &dados.autor,
        &dados.descricao,
        &dados.imagem, // 👈 base64 direto
        dados.latitude,
        dados.longitude,
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("dados.json")
        .expect("Erro ao abrir arquivo");

    writeln!(file, "{}", metadata).expect("Erro ao escrever");

    HttpResponse::Ok().json(Resposta { status: metadata })
}

// 🔥 LISTAR
async fn listar() -> impl Responder {
    let conteudo = read_to_string("dados.json").unwrap_or("".to_string());
    HttpResponse::Ok().body(conteudo)
}

// 🔥 METADATA
fn gerar_metadata(
    nome: &str,
    autor: &str,
    descricao: &str,
    imagem: &str,
    lat: f64,
    long: f64,
) -> String {
    serde_json::json!({
        "name": nome,
        "author": autor,
        "description": descricao,
        "image": imagem,
        "location": {
            "latitude": lat,
            "longitude": long
        }
    })
    .to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Content-Type"]);

        App::new()
            .wrap(cors)
            .app_data(web::JsonConfig::default().limit(10_000_000))
            .route("/registrar", web::post().to(registrar))
            .route("/listar", web::get().to(listar))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}