use actix_web::{HttpResponse, Result, http::header::ContentType,};
//use actix_web_flash_messages::IncomingFlashMessages;
use askama::Template; 

#[derive(Template)]
#[template(path = "login/home.html")]
struct HomeTemplate;

pub async fn home() -> Result<HttpResponse> {
    let template = HomeTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}
