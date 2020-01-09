use super::*;

use actix_web::body::{Body, ResponseBody};
use actix_web::dev::{Service, ServiceResponse};
use actix_web::http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use actix_web::test;
use std::collections::HashMap;

trait BodyTest {
    fn as_str(&self) -> &str;
}

#[derive(Debug, Deserialize, Serialize)]
struct ImgResponse {
    full: usize,
    thumb: usize,
}

impl BodyTest for ResponseBody<Body> {
    fn as_str(&self) -> &str {
        match self {
            ResponseBody::Body(ref b) => match b {
                Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                _ => panic!(),
            },
            ResponseBody::Other(ref b) => match b {
                Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                _ => panic!(),
            },
        }
    }
}

#[actix_rt::test]
async fn image_upload_remote_integration_test() {
    let mut app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::post()
        .uri("/upload/remote")
        .set_json(&ImgData {
            urls: vec![
                "https://randomuser.me/api/portraits/men/92.jpg".to_string(),
                "https://randomuser.me/api/portraits/men/34.jpg".to_string(),
                "https://randomuser.me/api/portraits/men/26.jpg".to_string(),
            ],
        })
        .to_request();
    let resp: ServiceResponse = app.call(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(CONTENT_TYPE).unwrap(),
        HeaderValue::from_static("application/json")
    );

    let map: HashMap<String, ImgResponse> =
        serde_json::from_str(resp.response().body().as_str()).unwrap();

    for key in map.keys() {
        let req = test::TestRequest::get()
            .uri(&format!("/image/original/{}", key))
            .to_request();
        let resp: ServiceResponse = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
