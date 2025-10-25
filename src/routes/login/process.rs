use actix_web::{
    error::InternalError,
    HttpResponse,
    http::header::ContentType,
    //http::header::LOCATION,
    web,
    Result,
};
//use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::{
    anyhow, Context
};
use serde::Deserialize;
use askama::Template;
//use uuid::Uuid;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordVerifier, Algorithm, Params, PasswordHasher, Version};
use secrecy::Secret;
use secrecy::ExposeSecret;
use crate::error::ApiError;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::session_state::TypedSession;

#[derive(Debug, Deserialize)]
pub struct LogInRequest {
    email: String,
    password: Secret<String>,
}

#[derive(Template)]
#[template(path = "login/success.html")]
struct LogInResponse {
    email: String,
    name: String,
    nickname: String,
}

#[tracing::instrument(
    name="Login Process(Validate Credentials)",
    skip(form, pool, session),
    fields(username=tracing::field::Empty, id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Json<LogInRequest>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<ApiError>> {
    let email = form.0.email;
    let expected_password = form.0.password;

    match login_process(&email, &pool).await {
        Ok(Some((email, name, nickname, password_hash, session_email))) => {
            //세션 정보 저장
            session.renew();
            session.insert_email(session_email).map_err(|e| login_redirect(ApiError::UnexpectError(e.into())))?;
            //비밀번호 체크
            spawn_blocking_with_tracing(move || {
                verify_password_hash(password_hash, expected_password)
            })
            .await
            .map_err(|e| login_redirect(ApiError::from(e)))?
            .map_err(|e| login_redirect(e))?;
            //템플릿 구조체로 데이터 저장
            let template = LogInResponse {
                email, name, nickname
            };
            //FromResidual 트레이트 : FromResidual 트레이트가 ? 연산자를 사용할 때 중요한 역할을 하는 트레이트이다. 에러 전파 또는 잔여(residual) 값을 상위 함수의 반환 타입으로 변환하는 방식을 정의
            let rendered = template.render().map_err(|e| {
                login_redirect(ApiError::from(e))
            })?;
            //서버가 HTML문자열을 응답 본문에 담아서 보내는 구문
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

#[tracing::instrument(
    name="Login Process(Session Email)",
    skip(email, pool, session),
    fields(username=tracing::field::Empty, id=tracing::field::Empty)
)]
pub async fn get_user_information(
    email: String,
    pool: &PgPool,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<ApiError>> {
    match login_process(&email, &pool).await {
        Ok(Some((email, name, nickname, _password_hash, session_email))) => {
            //세션 정보 저장
            session.renew();
            session.insert_email(session_email).map_err(|e| login_redirect(ApiError::UnexpectError(e.into())))?;
            //템플릿 구조체로 데이터 저장
            let template = LogInResponse {
                email, name, nickname
            };
            //FromResidual 트레이트 : FromResidual 트레이트가 ? 연산자를 사용할 때 중요한 역할을 하는 트레이트이다. 에러 전파 또는 잔여(residual) 값을 상위 함수의 반환 타입으로 변환하는 방식을 정의
            let rendered = template.render().map_err(|e| {
                login_redirect(ApiError::from(e))
            })?;
            //서버가 HTML문자열을 응답 본문에 담아서 보내는 구문
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
    email: &String,
    pool: &PgPool
) -> Result<Option<(String, String, String, Secret<String>, String)>, anyhow::Error> {
    tracing::debug!("email : {}", email);
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT email, name, nickname, password_hash, email AS session_email
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query")?
    .map(|row| (row.email, row.name, row.nickname, Secret::new(row.password_hash), row.session_email));
    
    Ok(row)
}

pub fn login_redirect(e: ApiError) -> InternalError<ApiError> {
    let response = HttpResponse::Unauthorized()
        .json(serde_json::json!({
            "error": e.to_string(),
            "redirect": "/home"
        }));
    InternalError::from_response(e, response)
}

#[derive(Template)]
#[template(path = "login/home.html")]
struct LogOutResponse;

pub async fn logout(session: TypedSession) -> Result<HttpResponse> {
    session.delete_email();
    let template = LogOutResponse;
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

#[tracing::instrument(
    name = "Verify password hash",
    skip(password_hash)
)]
fn verify_password_hash(
    password_hash: Secret<String>,
    expected_password: Secret<String>
) -> Result<(), ApiError> {
    let password_hash = PasswordHash::new(
        password_hash.expose_secret()
    )
    .context("Failed to parse hash in PHC string format")?;

    Argon2::default()
        .verify_password(expected_password.expose_secret().as_bytes(), &password_hash)
        .context("Invalid password. Please Enter Your Password Correctly.")
        .map_err(ApiError::InvalidCredentials)
}