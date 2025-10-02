use once_cell::sync::Lazy;
use rust_web::{
    configuration::{get_configuration, DatabaseSettings}, 
    startup::{get_connection_pool, Application}, 
    telemetry::{get_subscriber, init_subscriber}
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use chrono::Utc;


//'once_cell'을 사용해서 'TRACING' 스택이 한 번만 초기화되는 것을 보장한다.
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "into".to_string();
    
    //'get_subscriber'의 출력을 'TEST_LOG'의 값에 기반해서 변수에 할당할 수 없다.
    //왜냐하면 해당 sink는 'get_subscriber'에 의해 반환된 타입의 일부이고, 그들의 타입이 같지 않기 때문이다
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_filter_level, std::io::stdout, false);
        init_subscriber(subscriber);
    }else {
        //sink로 출력(로그 버림, 테스트 속도 향상)
        let subscriber = get_subscriber(default_filter_level, std::io::sink, false);
        init_subscriber(subscriber);
    }
});

//각 테스트를 위한 완전히 독립적인 애플리케이션 환경 생성
pub async fn spawn_app() -> TestApp {
    //'initialize'가 첫번째 호출되면 'TRACING'안의 코드가 실행된다. 다른 모든 호출은 실행을 건너뛴다.
    Lazy::force(&TRACING);

    let configuration =  {
        let mut c = get_configuration().expect("Failed to read configuration");
        //테스트 케이스마다 다른 데이터베이스 사용
        c.database.database_name = Uuid::new_v4().to_string();
        //무작위 OS 포트 사용
        c.application.port = 0;
        c
    };
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await.expect("Failed to build application");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        //port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        test_user: TestUser::generate(),
        api_client: client
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await.expect("Failed to connect to Postgres");
    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await.expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await.expect("Failed to connect to Posetgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await.expect("Failed to migrate the database");

    connection_pool
}

pub struct TestApp {
    //pub port: u16,
    pub address: String,
    pub db_pool: PgPool,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

impl TestApp {
    //로그인 엔드포인트에 POST 요청 / form() : URL-encoded 형식으로 전송
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where 
        Body: serde::Serialize, {
            self.api_client
                .post(&format!("{}/login", &self.address))
                .form(body)
                .send()
                .await
                .expect("Failed to execute request.")
        }
    //로그인 페이지 HTML 가져오기 / UI테스트나 CSRF토큰 추출 시 사용
    pub async fn get_login_html(&self) -> String {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
            .text()
            .await
            .unwrap()
    }
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub cntn: String
}

impl TestUser {
    //완전히 무작위 테스트 유저 생성 / UUID사용으로 충돌 없음 보장
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            cntn: Uuid::new_v4().to_string()
        }
    }
    /* 
    pub async fn login(&self, app: &TestApp) {
        app.post_login(&serde_json::json!({
            "username": &self.username,
            "cntn": &self.cntn
        }))
        .await;
    }
    */
    //생성된 테스트 유저를 실제 DB에 저장
    async fn store(&self, pool: &PgPool) {
        sqlx::query!(
            "INSERT INTO test_table (id, name, cntn, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)",
            self.user_id,
            self.username,
            self.cntn,
            Utc::now(),
            Utc::now()
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
    }
}

//작은 헬퍼 함수, 이번 창과 다음 창에서 이 확인을 여러 차례 수행한다.
pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}

/*
핵심
    -> 격리성: 각 테스트가 완전히 독립적인 DB와 서버 인스턴스 사용
    -> 병렬 실행 가능 : 포트/DB 충돌 없이 여러 테스트 동시 실행
    -> 실제 환경과 유사 : 진짜 HTTP 요청과 데이터베이스 사용
    -> 쿠키/세션 테스트 : 로그인 플로우 등 상태유지 시나리오 테스트 가능
    -> 디버깅 용이 : TEST_LOG 환경 변수로 필요시에만 로그 출력
*/