use actix_web::{
    error::InternalError,
    HttpResponse,
    http::header::LOCATION,
    web
};
use sqlx::PgPool;
use actix_web_flash_messages::FlashMessage;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::Context;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
}

#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field::Empty, id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let username = form.0.username;

    match login_process(&username, &pool).await {
        Ok(id) => {
            tracing::Span::current().record("id", &tracing::field::debug(id));
            Ok(HttpResponse::SeeOther().insert_header((LOCATION, "/login/succes")).finish())
        }
        Err(e) => { 
            let e = LoginError::AuthError(e.into());
            Err(login_redirect(e))
        }
    }
}

/*
cntn 반환타입이 Option<String>인 이유는 해당 컬럼이 Null값을 허용하기 때문이다.
*/
#[tracing::instrument(name="Login Process", skip(username, pool))]
async fn login_process(
    username: &String,
    pool: &PgPool
) -> Result<Option<(uuid::Uuid, String, Option<String>)>, anyhow::Error> {
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

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectError(#[from] anyhow::Error)
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

 pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();

    InternalError::from_response(e, response)
}