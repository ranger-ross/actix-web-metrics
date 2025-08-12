use std::collections::HashMap;

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_metrics::ActixWebMetricsBuilder;
use metrics_exporter_prometheus::PrometheusBuilder;

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Register a metrics exporter.
    // In this case we will just expose a Prometheus metrics endpoint on localhost:9000/metrics
    //
    // You can change this to another exporter based on your needs.
    // See https://github.com/metrics-rs/metrics for more info.
    PrometheusBuilder::new().install().unwrap();

    // Configure & build the Actix-Web middleware layer
    let mut labels = HashMap::new();
    labels.insert("label1".to_string(), "value1".to_string());
    let metrics = ActixWebMetricsBuilder::new().const_labels(labels).build();

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
