use actix_web::{http::header::ContentType, web, HttpResponse, Result};
//use actix_web_flash_messages::IncomingFlashMessages;
use askama::Template;
use sqlx::PgPool;
use crate::session_state::TypedSession;
use crate::routes::login::process::get_user_information;

#[derive(Template)]
#[template(path = "login/home.html")]
struct HomeTemplate;

pub async fn home_session(
    session: TypedSession,
    pool: web::Data<PgPool>
) -> Result<HttpResponse> {
    let email = session.get_email().unwrap_or(None).unwrap_or_default();

    if email.is_empty() {
        let template = HomeTemplate;
        let rendered = template.render().map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
    }else {
        get_user_information(&email, &pool).await.map_err(|e| e.into())
    }    
}
