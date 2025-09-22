use actix_web::{HttpResponse, http::header::ContentType};

pub async fn login_process() -> HttpResponse {
    HttpResponse::Ok().content_type(ContentType::html()).body(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>Hello!</title>
            </head>
            <body>
                <h1>Hello!</h1>
                <p>Hi from Rust</p>
            </body>
        </html>
        "#
    ))
}