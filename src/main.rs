use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::env;

#[derive(Serialize)]
struct Resposta {
    status: String,
}

async fn registrar(mut payload: Multipart) -> impl Responder {
    let mut nome = String::new();
    let mut descricao = String::new();
    let mut autor = String::new();
    let mut latitude = String::new();
    let mut longitude = String::new();
    let mut imagem_path = String::new();

    // cria pasta uploads se não existir
    fs::create_dir_all("./uploads").unwrap();

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        let content_disposition = field.content_disposition().unwrap();
        let name = content_disposition.get_name().unwrap();

        if name == "imagem" {
            let filename = format!("uploads/{}.jpg", uuid::Uuid::new_v4());
            let mut f = fs::File::create(&filename).unwrap();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).unwrap();
            }

            imagem_path = filename;
        } else {
            let mut value = Vec::new();
            while let Some(chunk) = field.next().await {
                value.extend_from_slice(&chunk.unwrap());
            }
            let text = String::from_utf8(value).unwrap();

            match name {
                "nome" => nome = text,
                "descricao" => descricao = text,
                "autor" => autor = text,
                "latitude" => latitude = text,
                "longitude" => longitude = text,
                _ => {}
            }
        }
    }

    // cria JSON do registro
    let registro = serde_json::json!({
        "nome": nome,
        "descricao": descricao,
        "autor": autor,
        "latitude": latitude,
        "longitude": longitude,
        "imagem": imagem_path
    });

    // salva no arquivo
    let mut dados = Vec::new();

    if let Ok(conteudo) = fs::read_to_string("dados.json") {
        if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&conteudo) {
            dados = json;
        }
    }

    dados.push(registro);

    fs::write("dados.json", serde_json::to_string_pretty(&dados).unwrap()).unwrap();

    HttpResponse::Ok().json(Resposta {
        status: "Registro salvo com sucesso".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // porta dinâmica (Render)
    let port = env::var("PORT").unwrap_or("8080".to_string());

    println!("🔥 SERVER STARTED on 0.0.0.0:{}", port);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            // 🔥 limite de upload (10MB)
            .app_data(web::PayloadConfig::new(10 * 1024 * 1024))

            .wrap(cors)

            // rota API
            .route("/registrar", web::post().to(registrar))

            // servir uploads
            .service(Files::new("/uploads", "./uploads").show_files_listing())

            // servir frontend
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}