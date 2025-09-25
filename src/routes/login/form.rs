use actix_web::{HttpResponse, Result};
use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;


pub async fn home() -> Result<HttpResponse> {
    let template = HomeTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type("text/html; charset=urf-8").body(rendered))
}