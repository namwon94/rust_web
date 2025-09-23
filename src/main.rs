use rust_web::{
    configuration::get_configuration,
    startup::Application,
};
use std::fmt::{Debug, Display};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 구성을 읽을 수 없으면 패닉에 빠진다
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    //여러 비동기 작업을 동시에 대기하고, 그중 가장 먼저 완료된 작업의 결과만 받아 처리한다. 
    tokio::select! {
        o = application_task => report_exit("API", o),
    };
    Ok(())
}

fn report_exit(
    task_name: &str,
    outcome: Result<Result<(), impl Debug + Display>, JoinError>
) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
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
