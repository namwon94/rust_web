use actix_web::{
    web, App, HttpServer, HttpResponse,
    dev::Server,
    Result,
    cookie::Key
};
use actix_web_flash_messages::{
    FlashMessagesFramework,
    storage::CookieMessageStore
};
use secrecy::{
    ExposeSecret,
    Secret,
};
//use actix_session::SessionMiddleware;
//use actix_session::storage::RedisSessionStore;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{
    contents, home, login, logout, register, registration
};
use askama::Template;

pub struct Application {
    port: u16,
    server: Server,
}

//생성, 실행, 종료 개념이 메서드로 분리되어 있으므로, 테스트 환경에서 각 단계별로 mocking 및 단위 테스트가 쉬워진다.
impl Application {
    //build 함수를 Application에 대한 생성자로 변환 / 비동기 함수이다. -> 초기화 실수 없이 안전하게 실행 환경을 만들 수 있다.
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let address = format!("{}:{}", configuration.application.host, configuration.application.port);
        //TCP 네트워크 서버를 구현할 때, 특정 IP주소와 포트로 들어오는 클라이언트의 TCP연결 요청을 받아들이고 대기하는 역할을 하는 표준 라이브러리의 구조체 이다.
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener, connection_pool, configuration.application.base_url, configuration.application.hmac_secret
        ).await?;

        Ok(Self{port, server})
    }

    //getter 등은 실제 서비스가 어떤 포트에 바인딩되었는지 외부에서 쉽게 참조할 수 있게 만들어, 테스트와 모듈 의존성 관리에도 유용하다.
    pub fn port(&self) -> u16 {
        self.port
    }

     // 이 함수는 애플리케이션이 중지되었을 때만 값을 반환한다는 것을 명확하게 나타내는 이름을 사용한다.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

//Actix Web의 web::Data로 등로되어 여러 핸들러에서 공유 가능한 상태로 만든다. 이 값을 통해 기본 URL(ex. APi 서버의 도메인) 정보를 전달한다.
pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener, db_pool: PgPool, base_url: String, hamc_secret: Secret<String>,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hamc_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    /*
    HttpServer::new 클로저 내에서 App::new()를 만들고 미들웨어, 라우트, 공유 상태를 설정한다.
    클로저를 인자로 받아 실행 하는 이유
        -> 서버가 시작될 때 합넙만 실행되는 것이 아니라, 요청이 들어올 때마다 필요한 설정을 새로 만들 수 있도록 설계
    move 클로저 : 외부에 잡아야 할 값이 있을 때 클로저 앞에 move가 붙는다. 클로저가 캡처하는 외부 변수들의 소유권을 가져가서 다른 스레드로 안전하게 전달할 수 있다.
    리다이렉션 대상인 "/hello.html" 경로에 대한 핸들러가 등록되어 있지 않으면, 클라이언트가 "/hello.html"로 다시 요청할 때 Actix Web이 해당 경로를 찾지 못해 404를 반환합니다.
    */
    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            //요청 로깅 미들웨어 추가
            .wrap(TracingLogger::default())
            //정적 파일
            .service(actix_files::Files::new("/css", "./static/css"))
            .service(actix_files::Files::new("/js", "./static/js"))
            .service(actix_files::Files::new("/templates", "./templates"))
            //동적 라우트
            .route("/", web::get().to(contents))
            //.route("/tracing_basic", web::get().to(tracing_basic))
            //404 처리
            .default_service(web::route().to(not_found))
            .route("/home", web::get().to(home))
            .route("/registration", web::get().to(registration))
            .route("/logout", web::post().to(logout))
            .route("/api/login", web::post().to(login))
            .route("/api/register", web::post().to(register))
            //DB풀과 베이스 URL정보를 애플리케이션 상태에 추가한다.
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    //acquire_timeout : 커넥션을 얻기 위한 최대 대기 시간 설정 / connect_lazy_with : 커넥션을 즉시 만들지 않고 필요 시 지연 연결하는 PgPool 객체를 생성한다. 
    PgPoolOptions::new().acquire_timeout(std::time::Duration::from_secs(2)).connect_lazy_with(configuration.with_db())
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate;

//404 처리 HTML
pub async fn not_found() -> Result<HttpResponse> {
    let template = NotFoundTemplate;
    let rendered = template.render().map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(rendered))
}