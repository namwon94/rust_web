use actix_web::{
    error::InternalError,
    HttpResponse,
    http::header::ContentType,
    http::header::LOCATION,
    web,
    Result,
};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::{
    anyhow, Context
};
use askama::Template;
use uuid::Uuid;
use argon2::password_hash::SaltString;
use argon2::{Argon2, Algorithm, Params, PasswordHasher, Version};
use secrecy::Secret;
use secrecy::ExposeSecret;
use crate::error::ApiError;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
}

#[derive(Template)]
#[template(path = "login/success.html")]
struct LoginProcess<'a> {
    id: Uuid,
    username: String,
    cntn: Option<&'a str>,
}

#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field::Empty, id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InternalError<ApiError>> {
    let username = form.0.username;

    match login_process(&username, &pool).await {
        Ok(Some((id, username, cntn))) => {
            let template = LoginProcess {
                id, username, cntn: cntn.as_deref()
            };
            //FromResidual 트레이트 : FromResidual 트레이트가 ? 연산자를 사용할 때 중요한 역할을 하는 트레이트이다. 에러 전파 또는 잔여(residual) 값을 상위 함수의 반환 타입으로 변환하는 방식을 정의
            let rendered = template.render().map_err(|e| {
                login_redirect(ApiError::from(e))
            })?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
        Ok(None) => {
            let e = ApiError::AuthError(anyhow!("No such user").into());
            Err(login_redirect(e))
        }
        Err(e) => { 
            Err(login_redirect(ApiError::from(e)))
        }
    }
}

/*
cntn 반환타입이 Option<String>인 이유는 해당 컬럼이 Null값을 허용하기 때문이다.
*/
#[tracing::instrument(name="Login Process", skip(pool))]
async fn login_process(
    username: &String,
    pool: &PgPool
) -> Result<Option<(uuid::Uuid, String, Option<String>)>, anyhow::Error> {
    tracing::debug!("username : {}", username);
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT id, name AS username, cntn
        FROM test_table
        WHERE name = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query")?
    .map(|row| (row.id, row.username, row.cntn));
    
    Ok(row)
}

pub fn login_redirect(e: ApiError) -> InternalError<ApiError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/home"))
        .finish();

    InternalError::from_response(e, response)
}

#[derive(Template)]
#[template(path = "login/home.html")]
struct LoginOut {
    message: String
}

pub async fn logout() -> Result<HttpResponse> {
    let message = String::new();
    let template = LoginOut{
        message,
    };
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
}

pub fn hash_password(
    password: &Secret<String>,
) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap()
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}