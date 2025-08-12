use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_metrics::ActixWebMetricsBuilder;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;

async fn health() -> HttpResponse {
    counter!("my_custom_counter").increment(1);
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    PrometheusBuilder::new().install().unwrap();

    let metrics = ActixWebMetricsBuilder::new().build();

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
