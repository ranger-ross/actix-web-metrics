use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_metrics::{ActixWebMetricsBuilder, ActixWebMetricsConfig};
use metrics_exporter_prometheus::PrometheusBuilder;

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    PrometheusBuilder::new().install().unwrap();

    let metrics = ActixWebMetricsBuilder::new()
        .metrics_config(
            ActixWebMetricsConfig::default()
                .http_requests_duration_seconds_name("my_http_request_duration"),
        )
        .build();

    HttpServer::new(move || {
        App::new()
            .wrap(metrics.clone())
            .service(web::resource("/health").to(health))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
    Ok(())
}
