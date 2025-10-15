use actix_web::{HttpResponse, Result, http::header::ContentType,};
use actix_web_flash_messages::IncomingFlashMessages;
use askama::Template; 

#[derive(Template)]
#[template(path = "login/home.html")]
struct HomeTemplate{
    message: String
}

pub async fn home(flash_message: IncomingFlashMessages) -> Result<HttpResponse> {
    let mut message = String::new();
    for m in flash_message.iter() {
        message = m.content().to_string();
    }
    let template = HomeTemplate{
        message,
    };
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}