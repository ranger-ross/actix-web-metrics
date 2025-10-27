/*!
[Metrics.rs](https://metrics.rs) integration for [actix-web](https://github.com/actix/actix-web).

This crate tries to adhere to [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/)

The following metrics are supported:

  - [`http.server.request.duration`](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#metric-httpserverrequestduration)
  - [`http.server.active_requests`](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#metric-httpserveractive_requests)
  - [`http.server.request.body.size`](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#metric-httpserverrequestbodysize)
  - [`http.server.response.body.size`](https://opentelemetry.io/docs/specs/semconv/http/http-metrics/#metric-httpserverresponsebodysize)


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
# if false {
    PrometheusBuilder::new().install().unwrap();
# }
    // Configure & build the Actix-Web middleware layer
    let metrics = ActixWebMetricsBuilder::new()
        .build();

# if false {
    HttpServer::new(move || {
        App::new()
            .wrap(metrics.clone())
            .service(web::resource("/health").to(health))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
# }
    Ok(())
}
```

In the example above we are using the `PrometheusBuilder` from the [metrics-exporter-prometheus](https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus) crate which exposes the metrics via an HTTP endpoint.

A call to the `localhost:9000/metrics` endpoint will expose your metrics:

```shell
$ curl http://localhost:9000/metrics

# HELP http_server_active_requests Number of active HTTP server requests.
# TYPE http_server_active_requests gauge
http_server_active_requests{http_request_method="GET",url_scheme="http"} 1

# HELP http_server_request_duration HTTP request duration in seconds for all requests
# TYPE http_server_request_duration summary
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0"} 0.000227207
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.5"} 0.00022719541927422382
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.9"} 0.00022719541927422382
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.95"} 0.00022719541927422382
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.99"} 0.00022719541927422382
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.999"} 0.00022719541927422382
http_server_request_duration{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="1"} 0.000227207
http_server_request_duration_sum{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1"} 0.000227207
http_server_request_duration_count{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1"} 1

# HELP http_server_response_body_size HTTP response size in bytes for all requests
# TYPE http_server_response_body_size summary
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0"} 0
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.5"} 0
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.9"} 0
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.95"} 0
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.99"} 0
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.999"} 0
http_server_response_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="1"} 0
http_server_response_body_size_sum{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1"} 0
http_server_response_body_size_count{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1"} 1

# HELP http_server_request_body_size HTTP request size in bytes for all requests
# TYPE http_server_request_body_size summary
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0"} 0
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.5"} 0
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.9"} 0
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.95"} 0
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.99"} 0
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="0.999"} 0
http_server_request_body_size{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1",quantile="1"} 0
http_server_request_body_size_sum{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1"} 0
http_server_request_body_size_count{http_route="/health",http_request_method="GET",http_response_status_code="200",network_protocol_name="http",network_protocol_version="1.1"} 1
```

NOTE: There are 2 important things to note:
* The `metrics-exporter-prometheus` crate can be swapped for another metrics.rs compatible exporter.
* The endpoint exposed by `metrics-exporter-prometheus` is not part of the actix web http server.

If you want to expose a prometheus endpoint directly in actix-web see the `prometheus_endpoint.rs` example.

# Features

## Custom metrics

The [metrics.rs](https://docs.rs/metrics/latest/metrics) crate provides macros for custom metrics.
This crate does interfere with that functionality.

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
        .build();

# if false {
        HttpServer::new(move || {
            App::new()
                .wrap(metrics.clone())
                .service(web::resource("/health").to(health))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
# }
    Ok(())
}
```

## Configurable routes pattern cardinality

Let's say you have on your app a route to fetch posts by language and by slug `GET /posts/{language}/{slug}`.
By default, actix-web-metrics will provide metrics for the whole route with the label `http_route` set to the pattern `/posts/{language}/{slug}`.
This is great but you cannot differentiate metrics across languages (as there is only a limited set of them).
Actix-web-metrics can be configured to allow for more cardinality on some route params.

For that you need to add a middleware to pass some [extensions data](https://blog.adamchalmers.com/what-are-extensions/), specifically the [`ActixWebMetricsExtension`] struct that contains the list of params you want to keep cardinality on.

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

If you want to rename the default metrics, you can use [`ActixWebMetricsConfig`] to do so.

```rust
use actix_web_metrics::{ActixWebMetricsBuilder, ActixWebMetricsConfig};

ActixWebMetricsBuilder::new()
    .metrics_config(
        ActixWebMetricsConfig::default()
           .http_server_request_duration_name("my_http_request_duration")
           .http_server_request_body_size_name("my_http_server_request_body_size_name")
           .http_server_response_body_size_name("my_http_server_response_body_size_name")
           .http_server_active_requests_name("my_http_server_active_requests_name"),
    )
    .build();
```

See full example `configuring_default_metrics.rs`.

## Masking unmatched requests

By default, if a request path is not matched to an Actix Web route, it will be masked as `UNKNOWN`.
This is useful to avoid producing lots of useless metrics due to bots or malious actors.

This can be configured in the following ways:
* `mask_unmatched_patterns()` can be used to change the `http_route` label to something other than `UNKNOWN`.
* `disable_unmatched_pattern_masking()` can be used to disable this masking functionality.

```rust,no_run
use actix_web_metrics::ActixWebMetricsBuilder;

ActixWebMetricsBuilder::new()
    .mask_unmatched_patterns("UNMATCHED")
    // or .disable_unmatched_pattern_masking()
    .build();
```

The above will convert all `/<nonexistent-path>` into `UNMATCHED`:

```text
http_requests_duration_seconds_sum{http_route="/favicon.ico",http_request_method="GET",http_response_status="400"} 0.000424898
```

becomes

```text
http_requests_duration_seconds_sum{http_route="UNMATCHED",http_request_method="GET",http_response_status="400"} 0.000424898
```
*/
#![deny(missing_docs)]

use actix_web::http::Uri;
use log::warn;
use metrics::{describe_gauge, describe_histogram, gauge, histogram, Unit};
use std::collections::{HashMap, HashSet};
use std::future::{ready, Future, Ready};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;

use actix_web::{
    body::{BodySize, MessageBody},
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::{Method, StatusCode, Version},
    web::Bytes,
    Error, HttpMessage,
};
use futures_core::ready;
use pin_project_lite::pin_project;

use regex::RegexSet;
use strfmt::strfmt;

/// ActixWebMetricsExtension define middleware and config struct to change the behaviour of the metrics
/// struct to define some particularities
#[derive(Debug, Clone)]
pub struct ActixWebMetricsExtension {
    /// list of params where the cardinality matters
    pub cardinality_keep_params: Vec<String>,
}

/// Builder to create new [`ActixWebMetrics`] struct.
#[derive(Debug)]
pub struct ActixWebMetricsBuilder {
    namespace: Option<String>,
    const_labels: HashMap<String, String>,
    exclude: HashSet<String>,
    exclude_regex: RegexSet,
    exclude_status: HashSet<StatusCode>,
    unmatched_patterns_mask: Option<String>,
    metrics_config: ActixWebMetricsConfig,
}

impl ActixWebMetricsBuilder {
    /// Create new `ActixWebMetricsBuilder`
    pub fn new() -> Self {
        Self {
            namespace: None,
            const_labels: HashMap::new(),
            exclude: HashSet::new(),
            exclude_regex: RegexSet::empty(),
            exclude_status: HashSet::new(),
            unmatched_patterns_mask: Some("UNKNOWN".to_string()),
            metrics_config: ActixWebMetricsConfig::default(),
        }
    }

    /// Set labels to add on every metrics
    pub fn const_labels(mut self, value: HashMap<String, String>) -> Self {
        self.const_labels = value;
        self
    }

    /// Set namespace
    pub fn namespace<T: Into<String>>(mut self, value: T) -> Self {
        self.namespace = Some(value.into());
        self
    }

    /// Ignore and do not record metrics for specified path.
    pub fn exclude<T: Into<String>>(mut self, path: T) -> Self {
        self.exclude.insert(path.into());
        self
    }

    /// Ignore and do not record metrics for paths matching the regex.
    pub fn exclude_regex<T: Into<String>>(mut self, path: T) -> Self {
        let mut patterns = self.exclude_regex.patterns().to_vec();
        patterns.push(path.into());
        self.exclude_regex = RegexSet::new(patterns).unwrap();
        self
    }

    /// Ignore and do not record metrics for paths returning the status code.
    pub fn exclude_status<T: Into<StatusCode>>(mut self, status: T) -> Self {
        self.exclude_status.insert(status.into());
        self
    }

    /// Replaces the request path with the supplied mask if no actix-web handler is matched
    ///
    /// Defaults to `UNKNOWN`
    pub fn mask_unmatched_patterns<T: Into<String>>(mut self, mask: T) -> Self {
        self.unmatched_patterns_mask = Some(mask.into());
        self
    }

    /// Disable masking of unmatched patterns.
    ///
    /// WARNING:This may lead to unbounded cardinality for unmatched requests. (potential DoS)
    pub fn disable_unmatched_pattern_masking(mut self) -> Self {
        self.unmatched_patterns_mask = None;
        self
    }

    /// Set metrics configuration
    pub fn metrics_config(mut self, value: ActixWebMetricsConfig) -> Self {
        self.metrics_config = value;
        self
    }

    /// Instantiate `ActixWebMetrics` struct
    ///
    /// WARNING: This call purposefully leaks the memory of metrics and label names to avoid
    /// allocations during runtime. Avoid calling more than once.
    pub fn build(self) -> ActixWebMetrics {
        let namespace_prefix = if let Some(ns) = self.namespace {
            format!("{ns}_")
        } else {
            "".to_string()
        };

        let http_server_request_duration_name = format!(
            "{namespace_prefix}{}",
            self.metrics_config.http_server_request_duration_name
        );
        describe_histogram!(
            http_server_request_duration_name.clone(),
            Unit::Seconds,
            "HTTP request duration in seconds for all requests"
        );

        let http_server_request_body_size_name = format!(
            "{namespace_prefix}{}",
            self.metrics_config.http_server_request_body_size_name
        );
        describe_histogram!(
            http_server_request_body_size_name.clone(),
            Unit::Bytes,
            "HTTP request size in bytes for all requests"
        );

        let http_server_response_body_size_name = format!(
            "{namespace_prefix}{}",
            self.metrics_config.http_server_response_body_size_name
        );
        describe_histogram!(
            http_server_response_body_size_name.clone(),
            Unit::Bytes,
            "HTTP response size in bytes for all requests"
        );

        let http_server_active_requests_name = format!(
            "{namespace_prefix}{}",
            self.metrics_config.http_server_active_requests_name
        );
        describe_gauge!(
            http_server_active_requests_name.clone(),
            "Number of active HTTP server requests."
        );

        let mut const_labels: Vec<(&'static str, String)> = self
            .const_labels
            .iter()
            .map(|(k, v)| {
                let k: &'static str = Box::leak(Box::new(k.clone()));
                (k, v.clone())
            })
            .collect();
        const_labels.sort_by_key(|v| v.0);

        ActixWebMetrics {
            inner: Arc::new(ActixWebMetricsInner {
                exclude: self.exclude,
                exclude_regex: self.exclude_regex,
                exclude_status: self.exclude_status,
                unmatched_patterns_mask: self.unmatched_patterns_mask,
                names: MetricsMetadata {
                    http_server_request_duration: Box::leak(Box::new(
                        http_server_request_duration_name,
                    )),
                    http_server_request_body_size: Box::leak(Box::new(
                        http_server_request_body_size_name,
                    )),
                    http_server_response_body_size: Box::leak(Box::new(
                        http_server_response_body_size_name,
                    )),
                    http_server_active_requests: Box::leak(Box::new(
                        http_server_active_requests_name,
                    )),
                    http_route: Box::leak(Box::new(self.metrics_config.labels.http_route)),
                    http_request_method: Box::leak(Box::new(
                        self.metrics_config.labels.http_request_method,
                    )),
                    http_response_status_code: Box::leak(Box::new(
                        self.metrics_config.labels.http_response_status_code,
                    )),
                    network_protocol_name: Box::leak(Box::new(
                        self.metrics_config.labels.network_protocol_name,
                    )),
                    network_protocol_version: Box::leak(Box::new(
                        self.metrics_config.labels.network_protocol_version,
                    )),
                    url_scheme: Box::leak(Box::new(self.metrics_config.labels.url_scheme)),
                    const_labels,
                },
            }),
        }
    }
}

impl Default for ActixWebMetricsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the labels used in metrics
#[derive(Debug, Clone)]
pub struct LabelsConfig {
    http_route: String,
    http_request_method: String,
    http_response_status_code: String,
    network_protocol_name: String,
    network_protocol_version: String,
    url_scheme: String,
}

impl Default for LabelsConfig {
    fn default() -> Self {
        Self {
            http_route: String::from("http.route"),
            http_request_method: String::from("http.request.method"),
            http_response_status_code: String::from("http.response.status_code"),
            network_protocol_name: String::from("network.protocol.name"),
            network_protocol_version: String::from("network.protocol.version"),
            url_scheme: String::from("url.scheme"),
        }
    }
}

impl LabelsConfig {
    /// set http method label
    pub fn http_request_method<T: Into<String>>(mut self, name: T) -> Self {
        self.http_request_method = name.into();
        self
    }

    /// set http route label
    pub fn http_route<T: Into<String>>(mut self, name: T) -> Self {
        self.http_route = name.into();
        self
    }

    /// set http status label
    pub fn http_response_status_code<T: Into<String>>(mut self, name: T) -> Self {
        self.http_response_status_code = name.into();
        self
    }

    /// set network protocol name label
    pub fn network_protocol_name<T: Into<String>>(mut self, name: T) -> Self {
        self.network_protocol_name = name.into();
        self
    }

    /// set network protocol version label
    pub fn network_protocol_version<T: Into<String>>(mut self, name: T) -> Self {
        self.network_protocol_version = name.into();
        self
    }

    /// set url scheme label
    pub fn url_scheme<T: Into<String>>(mut self, name: T) -> Self {
        self.url_scheme = name.into();
        self
    }
}

/// Configuration for the collected metrics
///
/// Stores individual metric configuration objects
#[derive(Debug, Clone)]
pub struct ActixWebMetricsConfig {
    http_server_request_duration_name: String,
    http_server_request_body_size_name: String,
    http_server_response_body_size_name: String,
    http_server_active_requests_name: String,
    labels: LabelsConfig,
}

impl Default for ActixWebMetricsConfig {
    fn default() -> Self {
        Self {
            http_server_request_duration_name: String::from("http.server.request.duration"),
            http_server_request_body_size_name: String::from("http.server.request.body.size"),
            http_server_response_body_size_name: String::from("http.server.response.body.size"),
            http_server_active_requests_name: String::from("http.server.active_requests"),
            labels: LabelsConfig::default(),
        }
    }
}

impl ActixWebMetricsConfig {
    /// Set the labels collected for the metrics
    pub fn labels(mut self, labels: LabelsConfig) -> Self {
        self.labels = labels;
        self
    }

    /// Set name for `http.server.request.duration` metric
    pub fn http_server_request_duration_name<T: Into<String>>(mut self, name: T) -> Self {
        self.http_server_request_duration_name = name.into();
        self
    }

    /// Set name for `http.server.request.body.size` metric
    pub fn http_server_request_body_size_name<T: Into<String>>(mut self, name: T) -> Self {
        self.http_server_request_body_size_name = name.into();
        self
    }

    /// Set name for `http.server.response.body.size` metric
    pub fn http_server_response_body_size_name<T: Into<String>>(mut self, name: T) -> Self {
        self.http_server_response_body_size_name = name.into();
        self
    }

    /// Set name for `http.server.active_requests` metric
    pub fn http_server_active_requests_name<T: Into<String>>(mut self, name: T) -> Self {
        self.http_server_active_requests_name = name.into();
        self
    }
}

/// Static references to variable metrics/label names.
/// This config primarily exists to avoid allocations during execution.
#[derive(Debug, Clone)]
struct MetricsMetadata {
    // metric names
    http_server_request_duration: &'static str,
    http_server_request_body_size: &'static str,
    http_server_response_body_size: &'static str,
    http_server_active_requests: &'static str,
    // label names
    http_route: &'static str,
    http_request_method: &'static str,
    http_response_status_code: &'static str,
    network_protocol_name: &'static str,
    network_protocol_version: &'static str,
    url_scheme: &'static str,
    const_labels: Vec<(&'static str, String)>,
}

/// An actix-web middleware the records metrics.
///
/// See the module documentation for more details
#[derive(Clone)]
#[must_use = "must be set up as middleware for actix-web"]
pub struct ActixWebMetrics {
    inner: Arc<ActixWebMetricsInner>,
}

struct ActixWebMetricsInner {
    pub(crate) names: MetricsMetadata,

    pub(crate) exclude: HashSet<String>,
    pub(crate) exclude_regex: RegexSet,
    pub(crate) exclude_status: HashSet<StatusCode>,
    pub(crate) unmatched_patterns_mask: Option<String>,
}

impl ActixWebMetrics {
    fn pre_request_update_metrics(&self, req: &ServiceRequest) {
        let this = &*self.inner;

        let mut labels = Vec::with_capacity(2 + this.names.const_labels.len());
        labels.push((
            this.names.http_request_method,
            req.method().as_str().to_string(),
        ));
        labels.push((this.names.url_scheme, url_scheme(&req.uri()).to_string()));
        for (k, v) in &this.names.const_labels {
            labels.push((k, v.clone()));
        }

        gauge!(this.names.http_server_active_requests, &labels).increment(1);
    }

    #[allow(clippy::too_many_arguments)]
    fn post_request_update_metrics(
        &self,
        http_version: Version,
        mixed_pattern: &str,
        fallback_pattern: &str,
        method: &Method,
        status: StatusCode,
        scheme: &str,
        clock: Instant,
        was_path_matched: bool,
        request_size: usize,
        response_size: usize,
    ) {
        let this = &*self.inner;

        // NOTE: active_requests cannot be skips as we need to decrement the increment we did that
        // the beginning of the request.
        {
            let mut active_request_labels = Vec::with_capacity(2 + this.names.const_labels.len());
            active_request_labels
                .push((this.names.http_request_method, method.as_str().to_string()));
            active_request_labels.push((this.names.url_scheme, scheme.to_string()));
            for (k, v) in &this.names.const_labels {
                active_request_labels.push((k, v.clone()));
            }
            gauge!(
                this.names.http_server_active_requests,
                &active_request_labels
            )
            .decrement(1);
        }

        if this.exclude.contains(mixed_pattern)
            || this.exclude_regex.is_match(mixed_pattern)
            || this.exclude_status.contains(&status)
        {
            return;
        }

        // do not record mixed patterns that were considered invalid by the server
        let final_pattern = if fallback_pattern != mixed_pattern && (status == 404 || status == 405)
        {
            fallback_pattern
        } else {
            mixed_pattern
        };

        let final_pattern = if was_path_matched {
            final_pattern
        } else if let Some(mask) = &this.unmatched_patterns_mask {
            mask
        } else {
            final_pattern
        };

        let mut labels = Vec::with_capacity(5 + this.names.const_labels.len());
        labels.push((this.names.http_route, final_pattern.to_string()));
        labels.push((this.names.http_request_method, method.as_str().to_string()));
        labels.push((
            this.names.http_response_status_code,
            status.as_str().to_string(),
        ));
        labels.push((this.names.network_protocol_name, "http".to_string()));

        if let Some(http_version) = Self::http_version_label(http_version) {
            labels.push((
                this.names.network_protocol_version,
                http_version.to_string(),
            ));
        }

        for (k, v) in &this.names.const_labels {
            labels.push((k, v.clone()));
        }

        let elapsed = clock.elapsed();
        let duration =
            (elapsed.as_secs() as f64) + f64::from(elapsed.subsec_nanos()) / 1_000_000_000_f64;
        histogram!(this.names.http_server_request_duration, &labels).record(duration);
        histogram!(this.names.http_server_request_body_size, &labels).record(request_size as f64);
        histogram!(this.names.http_server_response_body_size, &labels).record(response_size as f64);
    }

    fn http_version_label(version: Version) -> Option<&'static str> {
        let v = match version {
            v if v == Version::HTTP_09 => "0.9",
            v if v == Version::HTTP_10 => "1.0",
            v if v == Version::HTTP_11 => "1.1",
            v if v == Version::HTTP_2 => "2",
            v if v == Version::HTTP_3 => "3",
            _ => return None,
        };

        Some(v)
    }
}

impl<S, B> Transform<S, ServiceRequest> for ActixWebMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Response = ServiceResponse<StreamLog<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddleware {
            service,
            inner: self.clone(),
        }))
    }
}

pin_project! {
    #[doc(hidden)]
    pub struct LoggerResponse<S>
        where
        S: Service<ServiceRequest>,
    {
        #[pin]
        fut: S::Future,
        time: Instant,
        inner: ActixWebMetrics,
        _t: PhantomData<()>,
    }
}

impl<S, B> Future for LoggerResponse<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<StreamLog<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res = match ready!(this.fut.poll(cx)) {
            Ok(res) => res,
            Err(e) => return Poll::Ready(Err(e)),
        };

        let time = *this.time;
        let req = res.request();
        let method = req.method().clone();
        let version = req.version();
        let was_path_matched = req.match_pattern().is_some();

        // get metrics config for this specific route
        // piece of code to allow for more cardinality
        let params_keep_path_cardinality =
            match req.extensions_mut().get::<ActixWebMetricsExtension>() {
                Some(config) => config.cardinality_keep_params.clone(),
                None => vec![],
            };

        let full_pattern = req.match_pattern();
        let path = req.path().to_string();
        let fallback_pattern = full_pattern.clone().unwrap_or(path.clone());

        // mixed_pattern is the final path used as label value in metrics
        let mixed_pattern = match full_pattern {
            None => path.clone(),
            Some(full_pattern) => {
                let mut params: HashMap<String, String> = HashMap::new();

                for (key, val) in req.match_info().iter() {
                    if params_keep_path_cardinality.contains(&key.to_string()) {
                        params.insert(key.to_string(), val.to_string());
                        continue;
                    }
                    params.insert(key.to_string(), format!("{{{key}}}"));
                }

                if let Ok(mixed_cardinality_pattern) = strfmt(&full_pattern, &params) {
                    mixed_cardinality_pattern
                } else {
                    warn!("Cannot build mixed cardinality pattern {full_pattern}, with params {params:?}");
                    full_pattern
                }
            }
        };

        // Get request size from Content-Length header
        let request_size = req
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let scheme = url_scheme(&req.uri()).to_string();
        let inner = this.inner.clone();
        Poll::Ready(Ok(res.map_body(move |head, body| StreamLog {
            body,
            response_size: 0,
            request_size,
            clock: time,
            inner,
            status: head.status,
            scheme,
            mixed_pattern,
            fallback_pattern,
            method,
            version,
            was_path_matched,
        })))
    }
}

/// Middleware service for [`ActixWebMetrics`]
#[doc(hidden)]
pub struct MetricsMiddleware<S> {
    service: S,
    inner: ActixWebMetrics,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Response = ServiceResponse<StreamLog<B>>;
    type Error = S::Error;
    type Future = LoggerResponse<S>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        self.inner.pre_request_update_metrics(&req);

        LoggerResponse {
            fut: self.service.call(req),
            time: Instant::now(),
            inner: self.inner.clone(),
            _t: PhantomData,
        }
    }
}

pin_project! {
    #[doc(hidden)]
    pub struct StreamLog<B> {
        #[pin]
        body: B,
        response_size: usize,
        request_size: usize,
        clock: Instant,
        inner: ActixWebMetrics,
        status: StatusCode,
        scheme: String,
        // a route pattern with some params not-filled and some params filled in by user-defined
        mixed_pattern: String,
        fallback_pattern: String,
        method: Method,
        version: Version,
        was_path_matched: bool
    }


    impl<B> PinnedDrop for StreamLog<B> {
        fn drop(this: Pin<&mut Self>) {
            // update the metrics for this request at the very end of responding
            this.inner
                .post_request_update_metrics(this.version, &this.mixed_pattern, &this.fallback_pattern, &this.method, this.status, &this.scheme, this.clock, this.was_path_matched, this.request_size, this.response_size);
        }
    }
}

impl<B: MessageBody> MessageBody for StreamLog<B> {
    type Error = B::Error;

    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let this = self.project();
        match ready!(this.body.poll_next(cx)) {
            Some(Ok(chunk)) => {
                *this.response_size += chunk.len();
                Poll::Ready(Some(Ok(chunk)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}

fn url_scheme(uri: &Uri) -> &str {
    uri.scheme().map(|s| s.as_str()).unwrap_or("http")
}
