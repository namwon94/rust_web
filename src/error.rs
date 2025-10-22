use actix_web::HttpResponse;
use actix_web::http::header::LOCATION;

//#[derive(thiserror::Error)] : rust 표준 라이브러리의 std::error::Error트레이트 구현을 자동화한다.
#[derive(thiserror::Error)]
pub enum ApiError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Invalid Password.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectError(#[from] anyhow::Error),
    #[error("Template rendering error")]
    TemplateError(#[from] askama::Error),
    #[error("JoinHandle error")]
    JoinError(#[from] tokio::task::JoinError),
}

impl std::fmt::Debug for ApiError {
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

//로깅을 위해 오류의 근본 원인은 유지한면서 불투명한 500을 반환한다.
pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static {
        actix_web::error::ErrorInternalServerError(e)
    }

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther().insert_header((LOCATION, location)).finish()
}

//400을 반환한다. 바디에는 검증 오류에 대한 사용자 표현을 포함한다. 오류의 그본 원인은 로깅 목적을 위해 저장된다.
pub fn e400<T>(e: T) -> actix_web::Error
where 
    T: std::fmt::Debug + std::fmt::Display + 'static {
        actix_web::error::ErrorBadRequest(e)
    }