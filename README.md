# actix-web-metrics

[![CI Status](https://github.com/ranger-ross/actix-web-metrics/workflows/Test/badge.svg)](https://github.com/ranger-ross/actix-web-metrics/actions)
[![docs.rs](https://docs.rs/actix-web-metrics/badge.svg)](https://docs.rs/actix-web-metrics)
[![crates.io](https://img.shields.io/crates/v/actix-web-metrics.svg)](https://crates.io/crates/actix-web-metrics)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ranger-ross/actix-web-metrics/blob/master/LICENSE)

[Metrics.rs](https://metrics.rs) integration for [actix-web](https://github.com/actix/actix-web).

By default two metrics are tracked:

  - `http_requests_total` (labels: endpoint, method, status): the total number
    of HTTP requests handled by the actix HttpServer.

  - `http_requests_duration_seconds` (labels: endpoint, method, status): the
    request duration for all HTTP requests handled by the actix HttpServer.


# Usage

First add `actix-web-metrics` to your `Cargo.toml`:

```toml
[dependencies]
actix-web-metrics = "x.x.x"
```

You then instantiate the metrics middleware and pass it to `.wrap()`:

```rust
use std::collections::HashMap;

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_metrics::{ActixWebMetrics, ActixWebMetricsBuilder};
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
    let metrics = ActixWebMetricsBuilder::new()
        .build()
        .unwrap();

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
```

In the example above we are using the `PrometheusBuilder` from the [metrics-exporter-prometheus](https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus) crate which exposes the metrics via an HTTP endpoint.

A call to the `localhost:9000/metrics` endpoint will expose your metrics:

```shell
$ curl http://localhost:9000/metrics
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{endpoint="/health",method="GET",status="200",label1="value1"} 1

# HELP http_requests_duration_seconds HTTP request duration in seconds for all requests
# TYPE http_requests_duration_seconds summary
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="0"} 0.000302807
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="0.5"} 0.00030278122831198045
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="0.9"} 0.00030278122831198045
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="0.95"} 0.00030278122831198045
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="0.99"} 0.00030278122831198045
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="0.999"} 0.00030278122831198045
http_requests_duration_seconds{endpoint="/health",method="GET",status="200",label1="value1",quantile="1"} 0.000302807
http_requests_duration_seconds_sum{endpoint="/health",method="GET",status="200",label1="value1"} 0.000302807
http_requests_duration_seconds_count{endpoint="/health",method="GET",status="200",label1="value1"} 1
```

NOTE: There are 2 important things to note:
* The `metrics-exporter-prometheus` crate can be swapped for another metrics.rs compatible exporter.
* The endpoint exposed by `metrics-exporter-prometheus` is not part of the actix web http server.

If you want to expose a prometheus endpoint directly in actix-web see the `prometheus_endpoint.rs` example.

# Features

## Custom metrics

The [metrics.rs](https://docs.rs/metrics/latest/metrics) crate provides macros for custom metrics.
This crate does not interfere with that functionality.

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_metrics::{ActixWebMetrics, ActixWebMetricsBuilder};
use metrics::counter;

async fn health() -> HttpResponse {
    counter!("my_custom_counter").increment(1);
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let metrics = ActixWebMetricsBuilder::new()
        .build()
        .unwrap();

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
```

## Configurable routes pattern cardinality

Let's say you have on your app a route to fetch posts by language and by slug `GET /posts/{language}/{slug}`.
By default, actix-web-metrics will provide metrics for the whole route with the label `endpoint` set to the pattern `/posts/{language}/{slug}`.
This is great but you cannot differentiate metrics across languages (as there is only a limited set of them).
Actix-web-metrics can be configured to allow for more cardinality on some route params.

For that you need to add a middleware to pass some [extensions data](https://blog.adamchalmers.com/what-are-extensions/), specifically the [`MetricsConfig`] struct that contains the list of params you want to keep cardinality on.

```rust
use actix_web::{dev::Service, web, HttpMessage, HttpResponse};
use actix_web_metrics::ActixWebMetricsExtension;

async fn handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

web::resource("/posts/{language}/{slug}")
    .wrap_fn(|req, srv| {
        req.extensions_mut().insert::<ActixWebMetricsExtension>(
            ActixWebMetricsExtension { cardinality_keep_params: vec!["language".to_string()] }
        );
        srv.call(req)
    })
    .route(web::get().to(handler));
```

See the full example `with_cardinality_on_params.rs`.

## Configurable metric names

If you want to rename the default metrics, you can use `ActixWebMetricsConfig` to do so.

```rust
use actix_web_metrics::{ActixWebMetricsBuilder, ActixWebMetricsConfig};

ActixWebMetricsBuilder::new()
    .metrics_config(
        ActixWebMetricsConfig::default()
        .http_requests_duration_seconds_name("my_http_request_duration")
        .http_requests_duration_seconds_name("my_http_requests_duration_seconds"),
    )
    .build()
    .unwrap();
```

See full example `configuring_default_metrics.rs`.

## Masking unmatched requests

By default, if a request path is not matched to an Actix Web route, it will be masked as `UNKNOWN`.
This is useful to avoid producing lots of useless metrics due to bots or malicious actors.

This can be configured in the following ways:
* `mask_unmatched_patterns()` can be used to change the endpoint label to something other than `UNKNOWN`.
* `disable_unmatched_pattern_masking()` can be used to disable this masking functionality.

```rust,no_run
use actix_web_metrics::ActixWebMetricsBuilder;

ActixWebMetricsBuilder::new()
    .mask_unmatched_patterns("UNMATCHED")
    // or .disable_unmatched_pattern_masking()
    .build()
    .unwrap();
```

The above will convert all `/<nonexistent-path>` into `UNMATCHED`:

```text
http_requests_duration_seconds_sum{endpoint="/favicon.ico",method="GET",status="400"} 0.000424898
```

becomes

```text
http_requests_duration_seconds_sum{endpoint="UNMATCHED",method="GET",status="400"} 0.000424898
```

# Motivations

`actix-web-metrics` is heavily inspired (and forked from) [`actix-web-prom`](https://github.com/nlopes/actix-web-prom). 
Special thanks to @nlopes for their excellent work on `actix-web-prom`.

This crate replaces the underlying metrics implementation from the [`prometheus`](https://docs.rs/prometheus/latest/prometheus) crate with [`metrics.rs`](https://metrics.rs).

The reasons for doing this are as followed:

* The metrics.rs ecosystem provides more ergonomic ways to instrument applications than the raw prometheus client.
* `metrics.rs` provides more customizable ways to export metrics.
* The future of the `prometheus` crate is uncertain (see https://github.com/tikv/rust-prometheus/issues/530)

