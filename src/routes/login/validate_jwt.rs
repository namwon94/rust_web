use actix_web::{
    HttpRequest, HttpResponse, Result, cookie::{Cookie, SameSite, time::Duration}, error::InternalError, web
};
use sqlx::PgPool;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::anyhow;
use crate::{
    auth::JwtService, error::{ApiError, JwtError}, routes::login::process::{
        Credentials, LogInRequest, get_user_information_jwt, login_redirect, validate_email_query, verify_password_hash
    }, telemetry::spawn_blocking_with_tracing 
};

#[tracing::instrument(
    name="Validate Credentials(JWT)",
    skip(form, pool),
    fields(email=tracing::field::Empty, password=tracing::field::Empty)
)]
pub async fn validate_jwt(
    form: web::Form<LogInRequest>,
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>
) -> Result<HttpResponse, InternalError<ApiError>> {
    let credentials = Credentials {
        email: form.0.email,
        password: form.0.password
    };

    match validate_email_query(&credentials.email, &pool).await {
        Ok(Some((_email, password_hash))) => {
            //비밀번호 체크
            spawn_blocking_with_tracing(move || {
                verify_password_hash(password_hash, credentials.password)
            })
            .await
            .map_err(|e| login_redirect(ApiError::from(e)))?
            .map_err(|e| login_redirect(e))?;

            //jwt 토큰 생성
            let access_token = jwt_service.create_access_token(&credentials.email, Some("admin".to_string())).expect("Failed to load jwt(access)");
            let refresh_token = jwt_service.create_refresh_token(&credentials.email).expect("Faile to loat jwt(refresh)");

            let access_cookie = Cookie::build("access_token", access_token.clone())
                .path("/")
                .max_age(Duration::minutes(15))
                .http_only(true)
                .same_site(SameSite::Lax)
                .finish();
            let refresh_cookie = Cookie::build("refresh_token", refresh_token.clone())
                .path("/")
                .max_age(Duration::days(7))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .finish();
            //println!("access_token : {}, refresh_token : {}", access_token, refresh_token);
            //async fn은 호출 즉시 실행되지 않고 Future를 반환한다. 실제로 실행하려면 .await가 필요하다.
            let response = get_user_information_jwt(&credentials.email, &pool, Some(access_cookie), Some(refresh_cookie)).await?;

            Ok(response)

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

#[derive(Debug)]
pub enum CheckJwtToken {
    Guest,
    AccessValid {email: String},
    RefreshValid {
        email: String,
        access_cookie: Cookie<'static>,
        refresh_cookie: Cookie<'static>
    },
    InvalidToken
}

pub async fn check_token (
    req: &HttpRequest,
    jwt_service: &JwtService
) -> Result<CheckJwtToken, JwtError> {
    //1. access_token 시도
    //println!("jwt_service.extract_access_token(&req) : {:?}", jwt_service.extract_access_token(&req));
    if let Some(access_token) = jwt_service.extract_access_token(req) {
        //println!("acces_token verify start");
        match jwt_service.verify_access_token(&access_token) {
            Ok(claims) => return Ok(CheckJwtToken::AccessValid { email: claims.email }),
            Err(JwtError::ExpiredToken) => {

            }
            Err(JwtError::InvalidSignature) | Err(JwtError::InvalidToken) => {
                return Ok(CheckJwtToken::InvalidToken);
            }
            Err(_) => return Ok(CheckJwtToken::InvalidToken)
        }
    }
    //2. refresh_token 시도
    //println!("jwt_service.extract_refresh_token(&req) : {:?}", jwt_service.extract_refresh_token(&req));
    if let Some(refresh_token) = jwt_service.extract_refresh_token(req) {
        //println!("refresh_token verify start");
        match jwt_service.verify_refresh_token(&refresh_token) {
            Ok(claims) => {
                println!("claims(refresh) : {}", claims.email);
                let new_access_token = jwt_service.create_access_token(&claims.email, Some("admin".to_string())).expect("Faile to loat jwt(access)");
                let new_refresh_token = jwt_service.rotate_refresh_token(&refresh_token).expect("Faile to load jwt(refresh)");
                
                let access_cookie = Cookie::build("access_token", new_access_token.clone())
                .path("/")
                .max_age(Duration::minutes(15))
                .http_only(true)
                .same_site(SameSite::Lax)
                .finish();
                let refresh_cookie = Cookie::build("refresh_token", new_refresh_token.clone())
                    .path("/")
                    .max_age(Duration::days(7))
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Strict)
                    .finish();

                return Ok(CheckJwtToken::RefreshValid { email: claims.email, access_cookie, refresh_cookie })
            }
            Err(JwtError::ExpiredToken) => {
                println!("JwtError::ExpiredToken");
                return Ok(CheckJwtToken::InvalidToken);
            }
            Err(JwtError::InvalidSignature) | Err(JwtError::InvalidToken) => {
                println!("JwtError::InvalidSignature");
                return Ok(CheckJwtToken::InvalidToken);
            }
            Err(_) => return Ok(CheckJwtToken::InvalidToken)
        }
    }

    Ok(CheckJwtToken::Guest)
}