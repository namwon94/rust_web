use actix_web::{HttpResponse, Result, http::header::ContentType, web};
use actix_web_flash_messages::IncomingFlashMessages;
use askama::Template; 
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use secrecy::Secret;
use crate::routes::login::process::hash_password;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub nickname: String,
    pub password: Secret<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Template)]
#[template(path = "login/registration.html")]
struct RegisterTemplate{
    message: String
}

pub async fn registration(flash_message: IncomingFlashMessages) -> Result<HttpResponse> {
    let mut message = String::new();
    for m in flash_message.iter() {
        message = m.content().to_string();
    }
    let template = RegisterTemplate{
        message,
    };
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

#[tracing::instrument(
    name = "Register new user",
    skip(form, pool),
    fields (
        email = %form.email,
        nickname = %form.nickname
    )
)]

pub async fn register(
    form: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let password_hash = match hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().json(RegisterResponse{
                success: false,
                message: "비밀번호 처리 중 에러 발생".to_string()
            });
        }
    };

    match insert_user(&pool, &form.email, &form.name, &form.nickname, &password_hash).await {
        Ok(_) => HttpResponse::Ok().json(RegisterResponse {
            success: true,
            message: "회원 가입 성공".to_string()
        }),
        Err(e) => {
            tracing::error!("유저 회원가입 실패 : {:?}", e);

            let error_message = if e.to_string().contains("duplicate") {
                "이미 사용중인 이메일 있습니다."
            }else {
                "회원 가입 중 오류 발생했습니다."
            };

            HttpResponse::BadRequest().json(RegisterResponse {
                success: false,
                message: error_message.to_string(),
            })
        }
    }
}

async fn insert_user(
    pool: &PgPool,
    email: &str,
    name: &str,
    nickname: &str,
    password_hash: &str
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (email, name, nickname, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, now(), now())
        "#,
        email, name, nickname, password_hash
    )
    .execute(pool)
    .await?;

    Ok(())
}