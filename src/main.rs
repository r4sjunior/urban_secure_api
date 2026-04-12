use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use validator::Validate;
use log::info;
use std::env;

#[derive(Debug, Deserialize, Validate)]
struct Entrada {
    #[validate(length(min = 3, max = 100))]
    nome: String,

    #[validate(length(min = 5, max = 500))]
    descricao: String,

    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
struct Resposta {
    status: String,
}

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
        &dados.descricao,
        dados.latitude,
        dados.longitude,
    );

    HttpResponse::Ok().json(Resposta {
        status: metadata,
    })
}

fn gerar_metadata(nome: &str, descricao: &str, lat: f64, long: f64) -> String {
    serde_json::json!({
        "name": nome,
        "description": descricao,
        "location": {
            "latitude": lat,
            "longitude": long
        },
        "type": "arte urbana"
    })
    .to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // 🔥 PEGA PORTA DO RENDER (OBRIGATÓRIO)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();

    println!("🚀 Servidor rodando na porta {}", port);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin() // 🔥 libera acesso externo
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(web::JsonConfig::default().limit(4096))
            .route("/registrar", web::post().to(registrar))
    })
    .bind(("0.0.0.0", port))? // 🔥 MUITO IMPORTANTE
    .run()
    .await
}