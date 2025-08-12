use actix_web::{http::header::ContentType, web, App, HttpResponse, HttpServer};
use actix_web_metrics::ActixWebMetricsBuilder;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn get_metrics(prometheus: web::Data<PrometheusHandle>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(prometheus.render())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let prometheus = PrometheusBuilder::new().install_recorder().unwrap();

    let metrics = ActixWebMetricsBuilder::new().exclude("/metrics").build();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(prometheus.clone()))
            .wrap(metrics.clone())
            .service(web::resource("/metrics").get(get_metrics))
            .service(web::resource("/health").to(health))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
    Ok(())
}
