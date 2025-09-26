use rust_web::{
    configuration::get_configuration,
    startup::Application, 
    telemetry::{get_sbuscriber, init_subscriber},
};
use std::fmt::{Debug, Display};
//Tokio 런타임에서 비동기 작업(task)이 정상적으로 완료되지 못했을 때 발생하는 에러 타입 / tokio::spawn으로 생성한 작업이 내부적으로 panic이 발생하면 JoinError로 감싸져 호출자에게 반환
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //로그 출력 - level filter: expected one of "off", "error", "warn", "info", "debug", "trace", or a number 0-5
    let subscriber = get_sbuscriber("debug,sqlx::query=trace".into(), std::io::stdout);
    init_subscriber(subscriber);
    // 구성을 읽을 수 없으면 패닉에 빠진다
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    //tokio::spawn : Tokio런타임에서 비동기 작업(Future)을 백그라운드 태스크(경량 스레드)로 생성하고 실행하는 함수
    let application_task = tokio::spawn(application.run_until_stopped());

    //여러 비동기 작업을 동시에 대기하고, 그중 가장 먼저 완료된 작업의 결과만 받아 처리한다. 
    tokio::select! {
        o = application_task => report_exit("API", o),
    };
    Ok(())
}

//Tokio런타임에서 비동기 작업(task)의 종료 결과를 명확하게 처리하고 로깅하기 위한 함수
fn report_exit(
    //종료 결과를 기록할 작업(task)의 이름을 문자열로 받는다.
    task_name: &str,
    //Tokio의 JoinHandle::await로부터 받은 결과
    outcome: Result<Result<(), impl Debug + Display>, JoinError>
) {
    match outcome {
        //작업 성공
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        //작업은 정상 종료 하지만 작업 내에서 에러가 발생한 경우
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        //작업 실행 중에 Tokio가 작업을 완료하지 못하고 실패(JoinError)한 경우
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
/*
비동기 환경에서 작업의 성공 실패 여부를 안전하게 핸들링하고, 로그를 통해 문제를 투명하게 전달하는 역할을 수행.
체계적인 종료 처리 패턴은 대규모/복잡한 Rust 비동기 서버나 애플리케이션 개발 시 매우 유용.
*/
