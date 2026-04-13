use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Registro {
    titulo: String,
    descricao: String,
    imagem: String,
    latitude: f64,
    longitude: f64,
}

async fn salvar(mut payload: Multipart) -> impl Responder {
    let mut titulo = String::new();
    let mut descricao = String::new();
    let mut latitude = 0.0;
    let mut longitude = 0.0;
    let mut imagem_path = String::new();

    fs::create_dir_all("./uploads").unwrap();

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        let content = field.content_disposition().unwrap();
        let name = content.get_name().unwrap();

        if name == "imagem" {
            let filename = format!("./uploads/{}.jpg", uuid::Uuid::new_v4());
            let mut f = fs::File::create(&filename).unwrap();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).unwrap();
            }

            imagem_path = filename;
        } else {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk.unwrap());
            }

            let value = String::from_utf8(data).unwrap();

            match name {
                "titulo" => titulo = value,
                "descricao" => descricao = value,
                "latitude" => latitude = value.parse().unwrap_or(0.0),
                "longitude" => longitude = value.parse().unwrap_or(0.0),
                _ => {}
            }
        }
    }

    let novo = Registro {
        titulo,
        descricao,
        imagem: imagem_path,
        latitude,
        longitude,
    };

    let mut dados: Vec<Registro> = if let Ok(file) = fs::read_to_string("dados.json") {
        serde_json::from_str(&file).unwrap_or(vec![])
    } else {
        vec![]
    };

    dados.push(novo);

    fs::write("dados.json", serde_json::to_string_pretty(&dados).unwrap()).unwrap();

    HttpResponse::Ok().body("Salvo com sucesso")
}

async fn listar() -> impl Responder {
    let dados = fs::read_to_string("dados.json").unwrap_or("[]".to_string());
    HttpResponse::Ok().body(dados)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🔥 SERVER STARTED");

    HttpServer::new(|| {
        App::new()
            .route("/salvar", web::post().to(salvar))
            .route("/dados", web::get().to(listar))
    })
    .bind("0.0.0.0:10000")?
    .run()
    .await
}