use actix_web::{HttpResponse, Result, http::header::ContentType,};
use askama::Template; 

#[derive(Template)]
#[template(path = "ch1_1_basic.html")]
struct TracingBasicTemplate;

pub async fn tracing_basic() -> Result<HttpResponse> {
    let template = TracingBasicTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}