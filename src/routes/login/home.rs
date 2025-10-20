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
        println!("Flash message received: {}", m.content());
        message = m.content().to_string();
    }
    println!("message : {}",message);
    let template = HomeTemplate{
        message,
    };
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}