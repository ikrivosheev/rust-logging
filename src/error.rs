use actix_web::error::{JsonPayloadError, QueryPayloadError};
use actix_web::{error, HttpRequest, HttpResponse};

pub fn json_handler(err: JsonPayloadError, _: &HttpRequest) -> error::Error {
    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().body(detail),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => {
            HttpResponse::UnprocessableEntity().body(detail)
        }
        _ => HttpResponse::BadRequest().body(detail),
    };
    error::InternalError::from_response(err, resp).into()
}

pub fn query_handler(err: QueryPayloadError, _: &HttpRequest) -> error::Error {
    let detail = err.to_string();
    let resp = HttpResponse::BadRequest().body(detail);
    error::InternalError::from_response(err, resp).into()
}
