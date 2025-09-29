//트레이트 : 로그, 이벤트 등을 수집하는 기능을 정의한 추상 인터페이스
use tracing::Subscriber;
//모듈 : 전역 Subscriber 등록 함수
use tracing::subscriber::set_global_default;
use tracing_subscriber::{
    fmt::{self, time, MakeWriter, format::FmtSpan}, 
    layer::SubscriberExt, 
    registry::Registry, 
    EnvFilter 
};
use tracing_log::LogTracer;
use crate::configuration::Environment;
//use tokio::task::JoinHandle;
//BunyanFormattingLayer는 많은 메타데이터 필드를 포함하여 출력한다. 
//use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};

pub fn get_sbuscriber() -> Box<dyn Subscriber + Send + Sync> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    match environment {
        /*
        Box::new로 묶은 이유
            -> 각 함수의 impl Subscriber + Send + Sync 반환 타입은 **서로 다른 불투명 타입(opaque type)**으로 취급됩니다. 
                컴파일러 입장에서:
                    init_local_layer가 반환하는 impl Subscriber = 타입 A
                    init_production_layer가 반환하는 impl Subscriber = 타입 B
                비록 둘 다 Subscriber 트레이트를 구현하지만, 구체적인 타입이 다르기 때문에 match의 각 arm이 호환되지 않는 것입니다
         */
        Environment::Local => Box::new(init_local_layer("info,sqlx::query=trace".into(), std::io::stdout)),
        Environment::Production => Box::new(init_production_layer("info,sqlx::query=trace".into(), std::io::stdout)),
    }
}

/*
- tracing::Subscrbier : rust의 tracing 크레이트에서 트레이스 데이터를 수집하고 처리하는 핵심 트레이트
- Subscrber 트레이트 개념
    -> 트레이스 이벤트와 스팬을 수집하는 역할을 한다.
    -> 애플리케이션 실행 중 발생하는 이벤트 로그(함수 집입/종료, 로그 메시지, 메트릭 수집)를 처리한다.
    -> Subscriber는 트레이스 데이터를 다양한 대상으로 출력하거나 필터링하는 기능을 제공한다.

- 'impl Subscriber'를 반환 타입으로 사용해서 반환된 subscriber의 실제 타입에 관한 설명을 피한다.(매우 복잡하고 긴 타입이름이 됨.)
    -> 예를들어 하단에 Registry...이 여러 레이어를 조합한 결과 타입은 수백자가 넘는 복잡한 타입 시그니처가 된다.
    -> impl Trait를 반환타입을 사용하는 이유
        -> 구체적인 유형을 숨기고, 추상화된 인터페이스 타입만 반환해 함수 시그니처를 간결하게 유지하기 위함.
        -> 구체 타입의 변겨에 유연성을 제공하며, 다른 구현체로 쉽게 변경할 수 있다. 
- 반환된 subscriber를 'init_subscriber'로 나중에 전달하기 위해, 명시적으로 'Send'이고 'Sync'임을 알려야한다.
    -> Subscriber는 멀티스레드 환경에서 전역(global)으로 사용된다. 따라서 Send는 다른 스레드로 이동이 가능함을, Sync는 여러 스레드에서 동시에 접근 가능함을 의미한다.
    -> rust가 멀티스레드 안전성을 엄격히 검사하기 때문에 반환 타입에 Send + Sync제약을 명시해야 오류가 발생하지 않는다.
    -> tracgin같은 전역 로그 서브스크라이버는 스레드 세이프가 기본이다. 
*/
fn init_local_layer<Sink>(
    env_filter: String,
    //sink 타입 : tracing_subscriber에서 로그를 출력할 대상, 즉 로그가 기록될 "출력 채널" 역할을 하는 타입이다.
    sink: Sink
) -> impl Subscriber + Send + Sync + 'static
    where 
        /*
        - higer-ranked trait bound(HRTB) rust에서 lifetimes(생명주기)와 trait bound(트레이트 제약)를 더 일반적으로 표현하는 고급 개념. 
        - HRTB는 강력하게 모든 가능한 lifetime 'a에 대해 트레이트가 구현되어야 함을 의미 / for<'a> : for all litimes 'a라는 의미 
            -> 일반적인 트레이브 바운드 : 특정 lifetime 'a에 대해 타입이 트레이트를 구현한다고 명시
        - 기본적으로 Sink가 모든 라이프 타임 파라미터 'a'에 대해 MakeWriter 트레이트를 구현한다는 것을 의미
        */
        Sink: for<'a>MakeWriter<'a> + Send + Sync + 'static 
{
    //RUST_LOG 환경변수가 설정되어 있지 않으면 info 레벨 및 그 이상의 모든 span을 출력한다.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    Registry::default()
        .with(env_filter)
        .with(
            fmt::layer()
                //들여쓰기가 있는 읽기 쉬운 형식 - 프로덕션에서는 JSON, 개발에서는 pretty 권장
                .pretty()
                //ANSI색상 코드 활성화/비활성화
                .with_ansi(true)
                //시간 표시 
                .with_timer(time::UtcTime::rfc_3339())
                //로그가 발생한 모듈 경로 표시
                .with_level(true)
                //소스 코드 라인 번호
                .with_line_number(true)
                //소스 파일명
                .with_file(true)
                .with_target(true)
                //로그 레벨
                //스레드 이름 표시
                .with_thread_names(true)
                //스팬 관련 메서드
                //.with_span_events(FmtSpan::CLOSE)
                //스레드 ID 표시
                //.with_thread_ids(false)
                //출력 대상 메서드
                .with_writer(sink),
        )
    
}

fn init_production_layer<Sink>(
    env_filter: String,
    sink: Sink
) -> impl Subscriber + Send + Sync + 'static
    where 
        Sink: for<'a>MakeWriter<'a> + Send + Sync + 'static 
{
    //RUST_LOG 환경변수가 설정되어 있지 않으면 info 레벨 및 그 이상의 모든 span을 출력한다.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    Registry::default()
        .with(env_filter)
        .with(
            fmt::layer()
                //pretty(), json(), compact()
                .json()
                .with_timer(time::UtcTime::rfc_3339())
                .with_level(true)
                .with_line_number(true)
                .with_file(true)
                .with_target(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::CLOSE)
                .with_thread_ids(false)
                .with_writer(sink),
        )
}

//subscriber를 글로벌 기본값으로 등록해서 span데이터를 처리한다.(한 번만 호출)
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    //리다이렉트 처리
    LogTracer::init().expect("Failed to set logger");
    //애플리케이션에서 'set_global_default'를 사용해서 span을 처리하기 위해 어떤 subscriber를 사용해야 하는지 지정할 수 있다.
    set_global_default(subscriber).expect("Failed to set subscriber");
}