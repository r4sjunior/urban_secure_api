use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files::Files;
use actix_cors::Cors;
use serde::Deserialize;
use std::fs::{File, read_to_string};
use std::io::Write;
use uuid::Uuid;
use base64::{engine::general_purpose, Engine as _};
use serde_json;

#[derive(Deserialize)]
struct Arte {
    nome: String,
    descricao: String,
    latitude: f64,
    longitude: f64,
    imagem: String,
}

// SALVAR
async fn registrar(info: web::Json<Arte>) -> impl Responder {

    let base64_data = match info.imagem.split(",").nth(1) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("imagem inválida"),
    };

    let bytes = match general_purpose::STANDARD.decode(base64_data) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().body("erro base64"),
    };

    let filename = format!("/tmp/{}.png", Uuid::new_v4());

    let mut file = File::create(&filename).unwrap();
    file.write_all(&bytes).unwrap();

    // criar registro
    let registro = serde_json::json!({
        "nome": info.nome,
        "descricao": info.descricao,
        "latitude": info.latitude,
        "longitude": info.longitude,
        "imagem": filename
    });

    // ler banco
    let mut registros = Vec::new();

    if let Ok(conteudo) = read_to_string("/tmp/db.json") {
        registros = serde_json::from_str(&conteudo).unwrap_or(Vec::new());
    }

    registros.push(registro);

    let mut db = File::create("/tmp/db.json").unwrap();
    db.write_all(
        serde_json::to_string(&registros).unwrap().as_bytes()
    ).unwrap();

    HttpResponse::Ok().json("ok")
}

// LISTAR
async fn listar() -> impl Responder {
    match read_to_string("/tmp/db.json") {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(data),
        Err(_) => HttpResponse::Ok().body("[]"),
    }
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

            // ROTAS API
            .route("/registrar", web::post().to(registrar))
            .route("/listar", web::get().to(listar))

            // SERVIR IMAGENS
            .service(Files::new("/files", "/tmp").show_files_listing())

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