use secrecy::{ExposeSecret, Secret};
//serde와 함께 사용하는 '헬퍼 함수'로 JSON등에서 숫자 타입 필드를 문자열로 역직렬화(deserialize)할 때 사용된다.
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    ConnectOptions,
    postgres::{
        PgConnectOptions, PgSslMode,
    }
};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    //커넥션의 암호화 요청 여부를 결정한다.
    pub require_ssl: bool
}

//PgConnections는 DB연결 시 주로 사용된다. without_db는 DB선택 없이 서버 연결 설정만 하고, with_db는 해당 DB까지 지정해주는 기능
impl DatabaseSettings {
    //PgConnectOpions는 PostgreSQL 연결 설정을 표한하는 타입
    pub fn without_db(&self) -> PgConnectOptions {
        //ssl모드 설정
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        }else {
            //암호화된 커넥션을 시도한다. 실패하면 암호화하지 않는 커넥션을 사용한다.
            PgSslMode::Prefer
        };
        //체이닝으로 옵션을 구성한다. 체이닝(chaning) : 객체의 메서드들이 자기 자신 또는 또 다른 객체를 반환하도록 설계되어, 여러 메서드를 하나의 표현식으로 연속해서 호출할 수 있는 기법
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    //without_db()가 반환한 기본옵션에 데이터베이스이름을 추가해 완전한 연결 정보를 만든다. 실제 쿼리를 수행할 DB를 지정해주는 옵션을 반환한다. 또 로그레벨을 Trace로 설정하여 쿼리 실행 내역을 상세 로깅한다.
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name);

        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    //현재 디렉토리 출력
    let base_path = std::env::current_dir().expect("Failed to determin the current directory");
    let configuration_directory = base_path.join("configuration");
    //실행환경을 식별. 지정하지 않으면 'local'로 기본 설정
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        //Option이나 Result 타입에서 값이 없거나 오류일 때 대체 값을 제공하는 메서드이다.
        .unwrap_or_else(|_| "local".into())
        //한 타입을 다른 타입으로 변환을 시도할 때 사용한다. 이 변환은 실패할 수 있어서 반환값이 Result<T, E> 타입이며, 성공 시 변환된 값을, 실패 시 에러를 반환한다.
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());
    //base.yaml에서 구성값을 추가한다.
    let settings = config::Config::builder()
        .add_source(
            config::File::from(configuration_directory.join("base.yaml"))
        )
        .add_source(
            config::File::from(configuration_directory.join(&environment_filename))
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

//애플리케이션이 사용할 수 있는 런타임 환경 (##열거형##)
pub enum Environment {
    Local,
    Production
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either local or production.",
                other
            )),
        }
    }
}