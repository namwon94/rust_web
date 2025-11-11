use crate::telemetry::spawn_blocking_with_tracing;
use actix_web::{
    error::InternalError,
    HttpResponse,
    web,
    Result,
};
use sqlx::PgPool;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::{
    anyhow, Context
};
use secrecy::Secret;
use crate::{
    session_state::TypedSession,
    error::ApiError,
    routes::login::process::{
        LogInRequest, Credentials, verify_password_hash, login_redirect, get_user_information
    }
};



#[tracing::instrument(
    name="Validate Credentials",
    skip(form, pool, session),
    fields(username=tracing::field::Empty, id=tracing::field::Empty)
)]
pub async fn validate_credentials(
    form: web::Json<LogInRequest>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<ApiError>> {
    let credentials =  Credentials { 
        email: form.0.email, 
        password: form.0.password 
    };

    match validate_email_query(&credentials.email, &pool).await {
        Ok(Some((email,password_hash))) => {
            //비밀번호 체크
            spawn_blocking_with_tracing(move || {
                verify_password_hash(password_hash, credentials.password)
            })
            .await
            .map_err(|e| login_redirect(ApiError::from(e)))?
            .map_err(|e| login_redirect(e))?;
            //세션 정보 저장
            session.renew();
            session.insert_email(email).map_err(|e| login_redirect(ApiError::UnexpectError(e.into())))?;

            //async fn은 호출 즉시 실행되지 않고 Future를 반환한다. 실제로 실행하려면 .await가 필요하다.
            get_user_information(&credentials.email, &pool).await
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

#[tracing::instrument(name="Validate Email Query")]
async fn validate_email_query(
    email: &str,
    pool: &PgPool,
) -> Result<Option<(String, Secret<String>)>, anyhow::Error> {
    tracing::debug!("Email: {}", email);
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT email, password_hash
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query")?
    .map(|row| (row.email, Secret::new(row.password_hash)));

    Ok(row)
}
