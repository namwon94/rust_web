use actix_web::HttpRequest;
use actix_web::{http::header::ContentType, web, HttpResponse, Result};
//use actix_web_flash_messages::IncomingFlashMessages;
use askama::Template;
use sqlx::PgPool;
use crate::error::{e401, e500};
use crate::auth::{
    JwtService,
    TypedSession,
};
use crate::{
    routes::login::process::{
        get_user_information_session,
        get_user_information_jwt
    },
    error::ApiError
};

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
            e500(ApiError::InternalServerError(format!("InternalServerError : {}", e)))
        })?;
    
        Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
    }else {
        get_user_information_session(&email, &pool).await.map_err(|e| e.into())
    }    
}

pub async fn home_jwt(
    //HttpRequest는 모든 핸들러에서 자동으로 주입되는 타입이다.
    req: HttpRequest,
    jwt_service: web::Data<JwtService>,
    pool: web::Data<PgPool>
) -> Result<HttpResponse> {
    println!("jwt_service.extract_access_token(&req) : {:?}", jwt_service.extract_access_token(&req));
    match jwt_service.extract_access_token(&req) {
        Some(t) => {
            let claims = jwt_service.verify_access_token(&t)
                .map_err(|e| e401(ApiError::Unauthorized(format!("Invalid token : {}", e))))?;
            println!("claims.email : {}", claims.email);
            return get_user_information_jwt(&claims.email, &pool, None, None).await.map_err(|e| e.into())
        },
        None => {
            let template = HomeTemplate;
            let rendered = template.render().map_err(|e| {
                e500(ApiError::InternalServerError(format!("InternalServerError : {}", e)))
            })?;

            return Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
    };
}
