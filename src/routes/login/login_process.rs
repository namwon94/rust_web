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
//use actix_web_flash_messages::FlashMessage;
//anyhow의 확장 트레이트를 스코프 안으로 가져온다.
use anyhow::{
    anyhow, Context
};
use askama::Template;
use uuid::Uuid;

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
) -> Result<HttpResponse, InternalError<LoginError>> {
    let username = form.0.username;

    match login_process(&username, &pool).await {
        Ok(Some((id, username, cntn))) => {
            let template = LoginProcess {
                id, username, cntn: cntn.as_deref()
            };
            //FromResidual 트레이트 : FromResidual 트레이트가 ? 연산자를 사용할 때 중요한 역할을 하는 트레이트이다. 에러 전파 또는 잔여(residual) 값을 상위 함수의 반환 타입으로 변환하는 방식을 정의
            let rendered = template.render().map_err(|e| {
                let e = LoginError::TemplateError(e.into());
                login_redirect(e)
            })?;
            Ok(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))
        }
        Ok(None) => {
            let e = LoginError::AuthError(anyhow!("No such user").into());
            Err(login_redirect(e))
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

// #[derive(Template)]
// #[template(path = "login/redirect.html")]
// struct LoginRedirect {
//     error_message: String,
// }

// fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    
//     let template = LoginRedirect {
//         error_message: e.to_string()
//     };
//     //?를 사용할려면 반환 타입이 Result 혹은 Option 이어야 하는데 아니기 때문에 사용을 못한다. 그래서 macth로 명시적 에러처리를 했음.
//     let rendered = match template.render() {
//         Ok(s) => s,
//         Err(e) => {
//             let err = LoginError::TemplateError(e.into());
//             return InternalError::from_response(err, HttpResponse::InternalServerError().finish())
//         }
//     };
//     //FlashMessage::error(e.to_string()).send();
//     let response = HttpResponse::Ok().content_type(ContentType::html()).body(rendered);

//     InternalError::from_response(e, response)
// }
fn login_redirect(e: LoginError) -> InternalError<LoginError> {
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

//#[derive(thiserror::Error)] : rust 표준 라이브러리의 std::error::Error트레이트 구현을 자동화한다.
#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectError(#[from] anyhow::Error),
    #[error("Template rendering error")]
    TemplateError(#[from] askama::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

//std::error::Error의 source체인을 따라가면 원인(서브에러)까지 모두 출력한다.
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