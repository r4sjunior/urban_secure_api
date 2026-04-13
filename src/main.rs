use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files::Files;
use actix_cors::Cors;
use serde::Deserialize;
use std::fs::{File, read_to_string};
use std::io::Write;
use uuid::Uuid;
use base64::{engine::general_purpose, Engine as _};
use serde_json;

const DB_PATH: &str = "./data/db.json";
const IMG_DIR: &str = "./data/images";

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
    let id = Uuid::new_v4();
    let img_filename = format!("{}.png", id);
    let img_path = format!("{}/{}", IMG_DIR, img_filename);
    let img_url = format!("/files/{}", img_filename); // URL pública

    let base64_data = match info.imagem.split(",").nth(1) {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("imagem inválida"),
    };

    let bytes = match general_purpose::STANDARD.decode(base64_data) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().body("erro base64"),
    };

    // Garantir que o diretório de imagens existe
    if let Err(e) = std::fs::create_dir_all(IMG_DIR) {
        return HttpResponse::InternalServerError()
            .body(format!("erro ao criar diretório de imagens: {}", e));
    }

    // Salvar imagem
    let mut file = match File::create(&img_path) {
        Ok(f) => f,
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("erro ao criar arquivo de imagem: {}", e)),
    };

    if let Err(e) = file.write_all(&bytes) {
        return HttpResponse::InternalServerError()
            .body(format!("erro ao salvar imagem: {}", e));
    }

    // Criar registro com a URL pública da imagem
    let registro = serde_json::json!({
        "nome": info.nome,
        "descricao": info.descricao,
        "latitude": info.latitude,
        "longitude": info.longitude,
        "imagem": img_url
    });

    // Ler banco existente
    let mut registros: Vec<serde_json::Value> = Vec::new();
    if let Ok(conteudo) = read_to_string(DB_PATH) {
        registros = serde_json::from_str(&conteudo).unwrap_or_default();
    }

    registros.push(registro);

    // Garantir que o diretório de dados existe
    if let Err(e) = std::fs::create_dir_all("./data") {
        return HttpResponse::InternalServerError()
            .body(format!("erro ao criar diretório de dados: {}", e));
    }

    // Salvar banco atualizado
    let mut db = match File::create(DB_PATH) {
        Ok(f) => f,
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("erro ao abrir db.json: {}", e)),
    };

    if let Err(e) = db.write_all(serde_json::to_string(&registros).unwrap().as_bytes()) {
        return HttpResponse::InternalServerError()
            .body(format!("erro ao salvar db.json: {}", e));
    }

    HttpResponse::Ok().json("ok")
}

// LISTAR
async fn listar() -> impl Responder {
    match read_to_string(DB_PATH) {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .body(data),
        Err(_) => HttpResponse::Ok()
            .content_type("application/json")
            .body("[]"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Garantir que os diretórios existem ao iniciar
    std::fs::create_dir_all(IMG_DIR)?;
    std::fs::create_dir_all("./data")?;

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

            // SERVIR IMAGENS — aponta para ./data/images
            .service(Files::new("/files", IMG_DIR))

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