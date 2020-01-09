#[macro_use]
extern crate serde_derive;

use std::{
    fs,
    io::{self, prelude::Write},
    iter,
    path::Path,
};

use actix_multipart::Multipart;
use actix_web::{
    client::Client,
    middleware,
    web::{self, BytesMut},
    App, Error, HttpResponse, HttpServer,
};
use futures::StreamExt;
use rand::{self, distributions::Alphanumeric, Rng};

#[derive(Debug, Deserialize, Serialize)]
struct ImgData {
    urls: Vec<String>,
}

const IMAGE_FULL_PATH: &'static str = "storage/image/fullsize";
const IMAGE_THUMB_PATH: &'static str = "storage/image/thumbnail";

async fn image_create_preview(image_bytes: BytesMut, file_name: String) -> Result<usize, Error> {
    let img = image::load_from_memory(&image_bytes).unwrap();
    let scaled = img.thumbnail(100, 100);
    let file_path = format!("{}/{}", IMAGE_THUMB_PATH, file_name);
    let file_path_clone = file_path.clone();
    web::block(move || scaled.save(&file_path_clone)).await?;
    let file_size: usize = fs::metadata(&file_path)?.len() as usize;
    Ok(file_size)
}

async fn image_upload_local_request(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut file_sizes = String::new();
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_type = field.content_disposition().unwrap();
        let file_name = content_type.get_filename().unwrap();
        if file_name == "" {
            break;
        }
        let file_path = format!("{}/{}", IMAGE_FULL_PATH, file_name);
        let mut file = web::block(|| std::fs::File::create(file_path))
            .await
            .unwrap();
        let mut file_size: usize = 0;
        let mut file_bytes = BytesMut::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file_bytes.extend_from_slice(&data);
            file_size += data.len();
            file = web::block(move || file.write_all(&data).map(|_| file)).await?;
        }
        let thumb_size = image_create_preview(file_bytes, file_name.to_owned()).await?;
        file_sizes.push_str(&format!(
            "\"{}\": {{ \"full\": {}, \"thumb\": {} }}, ",
            file_name, file_size, thumb_size
        ));
    }
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("{{ {} }}", &file_sizes[..file_sizes.len() - 2]))
        .into())
}

async fn image_get_by_url(url: String) -> Result<BytesMut, Error> {
    let mut res = Client::default()
        .get(url)
        .send()
        .await
        .map_err(Error::from)?;

    let mut body = BytesMut::new();
    while let Some(chunk) = res.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(body)
}

async fn image_upload_remote_request(img_data: web::Json<ImgData>) -> Result<HttpResponse, Error> {
    let img_data = img_data.into_inner();
    let mut img_urls_iter = img_data.urls.iter();
    let mut file_sizes = String::new();
    while let Some(url) = img_urls_iter.next() {
        if url == "" {
            break;
        }
        let file_bytes = image_get_by_url(url.to_owned()).await?;
        let file_name: String = iter::repeat(())
            .map(|()| rand::thread_rng().sample(Alphanumeric))
            .take(9)
            .collect();
        let file_name = match Path::new(url).extension() {
            Some(ext) => match ext.to_str() {
                Some(ext) => format!("{}.{}", file_name, ext),
                None => file_name,
            },
            None => file_name,
        };
        let file_path = format!("{}/{}", IMAGE_FULL_PATH, file_name);
        let mut file = web::block(|| std::fs::File::create(file_path))
            .await
            .unwrap();
        let thumb_size = image_create_preview(file_bytes.clone(), file_name.to_owned()).await?;
        let file_size: usize = web::block(move || file.write(&file_bytes)).await?;
        file_sizes.push_str(&format!(
            "\"{}\": {{ \"full\": {}, \"thumb\": {} }}, ",
            file_name, file_size, thumb_size
        ));
    }
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("{{ {} }}", &file_sizes[..file_sizes.len() - 2]))
        .into())
}

fn app_config(config: &mut web::ServiceConfig) {
    std::fs::create_dir_all(IMAGE_FULL_PATH).unwrap();
    std::fs::create_dir_all(IMAGE_THUMB_PATH).unwrap();
    config.service(
        web::scope("")
            .data(Client::default())
            .service(
                web::resource("/upload/local").route(web::post().to(image_upload_local_request)),
            )
            .service(
                web::resource("/upload/remote").route(web::post().to(image_upload_remote_request)),
            )
            .service(actix_files::Files::new("/image/original/", IMAGE_FULL_PATH))
            .service(actix_files::Files::new("/image/preview/", IMAGE_THUMB_PATH))
            .service(actix_files::Files::new("/", "./static/").index_file("index.html")),
    );
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let endpoint = "127.0.0.1:30243";
    println!("Starting server at: {:?}", endpoint);
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind(endpoint)?
    .run()
    .await
}

#[cfg(test)]
mod tests;
