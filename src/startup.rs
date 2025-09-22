use actix_web::{
    web, App, HttpServer, HttpResponse,
    dev::Server,
    http::header::ContentType,
};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::routes::{
    login_process,
};


pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            /*
            리다이렉션 대상인 "/hello.html" 경로에 대한 핸들러가 등록되어 있지 않으면, 클라이언트가 "/hello.html"로 다시 요청할 때 Actix Web이 해당 경로를 찾지 못해 404를 반환합니다.
             */
            .route("/", web::get().to(login_process))
            //.route("/", web::get().to(|| async { "Hello, world!" }))
            //404 처리
            .default_service(web::route().to(not_found))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().content_type(ContentType::html()).body(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>Error!</title>
            </head>
            <body>
                <h1>Error!</h1>
                <p>404 - 페이지를 찾을 수 없습니다</p>
                <p>Sorry, I don't know what you're asking for.</p>
            </body>
        </html>
        "#
    ))
}