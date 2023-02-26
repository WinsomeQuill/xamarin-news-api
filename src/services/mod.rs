use std::str::FromStr;
use actix_web::web::Payload;
use actix_web::{HttpRequest, web};
use actix_web::web::BytesMut;
use serde::Serialize;
use serde_json::{Value, json};
use futures::StreamExt;
use qstring::QString;

const MAX_SIZE_BUFFER_REQUEST: usize = 16_777_216; // максимальный размер буфера - 256кб

pub(crate) fn json_error<T>(message: T) -> Value
where T: Serialize {
    json!({
        "status": "error",
        "message": message
    })
}

pub(crate) fn json_success<T>(message: T) -> Value
where T: Serialize {
    json!({
        "status": "success",
        "message": message
    })
}

pub(crate) async fn read_body_bytes(payload: &mut Payload) -> Result<BytesMut, ()> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        if (body.len() + chunk.len()) > MAX_SIZE_BUFFER_REQUEST {
            return Err(());
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

pub(crate) async fn get_query_param<T: FromStr>(req: &HttpRequest, query_key: &str) -> Result<T, String> {
    let query_str = req.query_string();
    let qs = QString::from(query_str);

    let result = match qs.get(query_key) {
        Some(o) => match o.parse::<T>() {
            Ok(o) => o,
            Err(_) => return Err(format!("Invalid query type for {}!", &query_key)),
        },
        None => return Err(format!("Not found {}!", &query_key)),
    };

    Ok(result)
}

pub mod service_user;
pub mod service_article;