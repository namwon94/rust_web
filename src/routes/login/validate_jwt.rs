use actix_web::{
    HttpResponse, Result, cookie::{Cookie, SameSite, time::Duration}, error::InternalError, web
};
use sqlx::PgPool;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::anyhow;
use crate::{
    //session_state::TypedSession,
    error::ApiError, 
    auth::JwtService, 
    telemetry::spawn_blocking_with_tracing,
    routes::login::process::{
        Credentials, LogInRequest, get_user_information_jwt, login_redirect, validate_email_query, verify_password_hash
    }, 
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