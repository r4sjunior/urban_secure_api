use actix_web::{post, App, HttpServer, Responder, HttpResponse};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use actix_cors::Cors;

#[post("/registrar")]
async fn registrar(mut payload: Multipart) -> impl Responder {
    println!("Recebendo upload...");

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                println!("Erro ao ler campo: {:?}", e);
                return HttpResponse::BadRequest().body("Erro no upload");
            }
        };

        // 👇 pega metadata SEM causar conflito de borrow
        let field_name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .unwrap_or("")
            .to_string();

        println!("Campo: {}", field_name);

        let mut data = Vec::new();

        // 👇 leitura do arquivo/dados
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(bytes) => data.extend_from_slice(&bytes),
                Err(e) => {
                    println!("Erro ao ler chunk: {:?}", e);
                    return HttpResponse::InternalServerError().body("Erro ao processar upload");
                }
            }
        }

        println!("Recebido {} bytes no campo {}", data.len(), field_name);

        // 👇 tratamento dos campos
        match field_name.as_str() {
            "file" => {
                println!("Arquivo recebido ({} bytes)", data.len());

                // exemplo: salvar arquivo (opcional)
                // std::fs::write("upload.bin", &data).unwrap();
            }
            "nome" => {
                let texto = String::from_utf8_lossy(&data);
                println!("Nome: {}", texto);
            }
            _ => {
                println!("Campo desconhecido: {}", field_name);
            }
        }
    }

    HttpResponse::Ok().body("Upload recebido com sucesso")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Servidor rodando na porta 10000");

    HttpServer::new(|| {
        App::new()
            // 👇 LIMITE DE UPLOAD (resolve 413)
            .app_data(actix_web::web::PayloadConfig::new(50 * 1024 * 1024)) // 50MB

            // 👇 libera acesso do frontend
            .wrap(Cors::permissive())

            // 👇 rota
            .service(registrar)
    })
    .bind(("0.0.0.0", 10000)) // obrigatório no Render
    .unwrap()
    .run()
    .await
}