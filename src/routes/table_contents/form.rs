use actix_web::{HttpResponse, Result, http::header::ContentType,};
use askama::Template; 

#[derive(Template)]
#[template(path = "table_contents.html")]
struct TableContentsTemplate;

pub async fn contents() -> Result<HttpResponse> {
    let template = TableContentsTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}