use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use std::io::Write;
use uuid::Uuid;

// =======================
// 📌 ROTA DE UPLOAD
// =======================
async fn salvar(mut payload: Multipart) -> impl Responder {
    let mut nome = String::new();
    let mut descricao = String::new();
    let mut latitude = String::new();
    let mut longitude = String::new();
    let mut imagem_path = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        let content_disposition = field.content_disposition();

        let field_name = content_disposition
            .get_name()
            .unwrap_or("");

        // =======================
        // 📸 UPLOAD IMAGEM
        // =======================
        if field_name == "imagem" {
            let filename = format!("{}.jpg", Uuid::new_v4());
            let filepath = format!("./uploads/{}", filename);

            let mut f = std::fs::File::create(&filepath).unwrap();

            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                f.write_all(&data).unwrap();
            }

            imagem_path = filepath;
        } 
        // =======================
        // 📝 CAMPOS TEXTO
        // =======================
        else {
            let mut bytes = Vec::new();

            while let Some(chunk) = field.next().await {
                bytes.extend_from_slice(&chunk.unwrap());
            }

            let text = String::from_utf8(bytes).unwrap_or_default();

            match field_name {
                "nome" => nome = text,
                "descricao" => descricao = text,
                "latitude" => latitude = text,
                "longitude" => longitude = text,
                _ => {}
            }
        }
    }

    println!("📍 NOVO REGISTRO");
    println!("Nome: {}", nome);
    println!("Descrição: {}", descricao);
    println!("Lat: {}", latitude);
    println!("Lng: {}", longitude);
    println!("Imagem: {}", imagem_path);

    HttpResponse::Ok().body("Salvo com sucesso 🚀")
}

// =======================
// 🚀 MAIN
// =======================
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // cria pasta uploads automaticamente
    std::fs::create_dir_all("./uploads").unwrap();

    println!("🔥 SERVER STARTED on 0.0.0.0:10000");

    HttpServer::new(|| {
        App::new()
            // 🔥 AUMENTA LIMITE DE UPLOAD (resolve erro 413)
            .app_data(web::PayloadConfig::new(50 * 1024 * 1024))

            // 🌐 libera CORS
            .wrap(Cors::permissive())

            // 📡 API
            .route("/registrar", web::post().to(salvar))

            // 🌍 FRONTEND
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
            )
    })
    .bind(("0.0.0.0", 10000))?
    .run()
    .await
}