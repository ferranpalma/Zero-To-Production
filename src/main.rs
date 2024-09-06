use rust_zero2prod::{configuration, startup, telemetry};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");

    let tracing_subsriber =
        telemetry::get_tracing_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_tracing_subscriber(tracing_subsriber);

    let server = startup::Application::build_application(&configuration).await?;
    server.run_server().await?;

    Ok(())
}
