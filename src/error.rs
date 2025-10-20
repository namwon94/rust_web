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