use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build_application(
        configuration: &Settings,
    ) -> Result<Application, std::io::Error> {
        let db_connection_pool = get_db_connection_pool(&configuration.database);

        let email_client_timeout = configuration.email_client.get_timeout();
        let email_client = EmailClient::new(
            &configuration.email_client.base_url,
            configuration.email_client.sender_email.clone(),
            configuration.email_client.api_token.clone(),
            email_client_timeout,
        );

        let server_address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let server_tcp_socket = TcpListener::bind(server_address)?;
        let server_port = server_tcp_socket.local_addr().unwrap().port();

        let server = Self::build_http_server(server_tcp_socket, db_connection_pool, email_client)?;

        Ok(Self {
            port: server_port,
            server,
        })
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub async fn run_server(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    fn build_http_server(
        tcp_socket: TcpListener,
        db_connection_pool: PgPool,
        email_client: EmailClient,
    ) -> Result<Server, std::io::Error> {
        let db_connection_pool = web::Data::new(db_connection_pool);
        let http_email_client = web::Data::new(email_client);
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .app_data(db_connection_pool.clone())
                .app_data(http_email_client.clone())
                .service(health_check)
                .service(subscribe)
        })
        .listen(tcp_socket)?
        .run();

        Ok(server)
    }
}

pub fn get_db_connection_pool(db_settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(db_settings.get_connect_options())
}
