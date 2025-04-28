/*!
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
# if false {
    PrometheusBuilder::new().install().unwrap();
# }
    // Configure & build the Actix-Web middleware layer
    let metrics = ActixWebMetricsBuilder::new()
        .build()
        .unwrap();

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
        .build()
        .unwrap();

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
By default, actix-web-metrics will provide metrics for the whole route with the label `endpoint` set to the pattern `/posts/{language}/{slug}`.
This is great but you cannot differentiate metrics across languages (as there is only a limited set of them).
Actix-web-metrics can be configured to allow for more cardinality on some route params.

For that you need to add a middleware to pass some [extensions data](https://blog.adamchalmers.com/what-are-extensions/), specifically the [`MetricsConfig`] struct that contains the list of params you want to keep cardinality on.

```rust
use actix_web::{dev::Service, web, HttpMessage, HttpResponse};
use actix_web_metrics::MetricsConfig;

async fn handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

web::resource("/posts/{language}/{slug}")
    .wrap_fn(|req, srv| {
        req.extensions_mut().insert::<MetricsConfig>(
            MetricsConfig { cardinality_keep_params: vec!["language".to_string()] }
        );
        srv.call(req)
    })
    .route(web::get().to(handler));
```

See the full example `with_cardinality_on_params.rs`.

## Configurable metric names

If you want to rename the default metrics, you can use [`ActixMetricsConfiguration`] to do so.

```rust
use actix_web_metrics::{ActixWebMetricsBuilder, ActixMetricsConfiguration};

ActixWebMetricsBuilder::new()
    .metrics_configuration(
        ActixMetricsConfiguration::default()
        .http_requests_duration_seconds_name("my_http_request_duration")
        .http_requests_duration_seconds_name("my_http_requests_duration_seconds"),
    )
    .build()
    .unwrap();
```

See full example `configuring_default_metrics.rs`.

## Masking unmatched requests

By default, if a request path is not matched to an Actix Web route, it will be masked as `UNKNOWN`.
This is useful to avoid producing lots of useless metrics due to bots or malious actors.

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
*/
#![deny(missing_docs)]

use log::warn;
use metrics::{counter, describe_counter, describe_histogram, histogram};
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

/// MetricsConfig define middleware and config struct to change the behaviour of the metrics
/// struct to define some particularities
#[derive(Debug, Clone)]
pub struct MetricsConfig {
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
    metrics_configuration: ActixMetricsConfiguration,
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
            metrics_configuration: ActixMetricsConfiguration::default(),
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
    pub fn metrics_configuration(mut self, value: ActixMetricsConfiguration) -> Self {
        self.metrics_configuration = value;
        self
    }

    /// Instantiate `ActixWebMetrics` struct
    ///
    /// WARNING: This call purposefully leaks the memory of metrics and label names to avoid
    /// allocations during runtime. Avoid calling more than once.
    pub fn build(self) -> Result<ActixWebMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let namespace_prefix = if let Some(ns) = self.namespace {
            format!("{ns}_")
        } else {
            "".to_string()
        };

        let http_requests_duration_seconds_name = format!(
            "{namespace_prefix}{}",
            self.metrics_configuration
                .http_requests_duration_seconds_name
        );
        describe_histogram!(
            http_requests_duration_seconds_name.clone(),
            "HTTP request duration in seconds for all requests"
        );
        let http_requests_total_name = format!(
            "{namespace_prefix}{}",
            self.metrics_configuration.http_requests_total_name
        );
        describe_counter!(
            http_requests_total_name.clone(),
            "Total number of HTTP requests"
        );

        let version: Option<&'static str> =
            if let Some(ref v) = self.metrics_configuration.labels.version {
                Some(Box::leak(Box::new(v.clone())))
            } else {
                None
            };

        let mut const_labels: Vec<(&'static str, String)> = self
            .const_labels
            .iter()
            .map(|(k, v)| {
                let k: &'static str = Box::leak(Box::new(k.clone()));
                (k, v.clone())
            })
            .collect();
        const_labels.sort_by_key(|v| v.0);

        Ok(ActixWebMetrics {
            exclude: self.exclude,
            exclude_regex: self.exclude_regex,
            exclude_status: self.exclude_status,
            enable_http_version_label: self.metrics_configuration.labels.version.is_some(),
            unmatched_patterns_mask: self.unmatched_patterns_mask,
            names: MetricsMetadata {
                http_requests_total: Box::leak(Box::new(http_requests_total_name)),
                http_requests_duration_seconds: Box::leak(Box::new(
                    http_requests_duration_seconds_name,
                )),
                endpoint: Box::leak(Box::new(self.metrics_configuration.labels.endpoint)),
                method: Box::leak(Box::new(self.metrics_configuration.labels.method)),
                status: Box::leak(Box::new(self.metrics_configuration.labels.status)),
                version,
                const_labels,
            },
        })
    }
}

///Configurations for the labels used in metrics
#[derive(Debug, Clone)]
pub struct LabelsConfiguration {
    endpoint: String,
    method: String,
    status: String,
    version: Option<String>,
}

impl Default for LabelsConfiguration {
    fn default() -> Self {
        Self {
            endpoint: String::from("endpoint"),
            method: String::from("method"),
            status: String::from("status"),
            version: None,
        }
    }
}

impl LabelsConfiguration {
    /// set http method label
    pub fn method<T: Into<String>>(mut self, name: T) -> Self {
        self.method = name.into();
        self
    }

    /// set http endpoint label
    pub fn endpoint<T: Into<String>>(mut self, name: T) -> Self {
        self.endpoint = name.into();
        self
    }

    /// set http status label
    pub fn status<T: Into<String>>(mut self, name: T) -> Self {
        self.status = name.into();
        self
    }

    /// set http version label
    pub fn version<T: Into<String>>(mut self, name: T) -> Self {
        self.version = Some(name.into());
        self
    }
}

/// Configuration for the collected metrics
///
/// Stores individual metric configuration objects
#[derive(Debug, Clone)]
pub struct ActixMetricsConfiguration {
    http_requests_total_name: String,
    http_requests_duration_seconds_name: String,
    labels: LabelsConfiguration,
}

impl Default for ActixMetricsConfiguration {
    fn default() -> Self {
        Self {
            http_requests_total_name: String::from("http_requests_total"),
            http_requests_duration_seconds_name: String::from("http_requests_duration_seconds"),
            labels: LabelsConfiguration::default(),
        }
    }
}

impl ActixMetricsConfiguration {
    /// Set the labels collected for the metrics
    pub fn labels(mut self, labels: LabelsConfiguration) -> Self {
        self.labels = labels;
        self
    }

    /// Set name for `http_requests_total` metric
    pub fn http_requests_total_name<T: Into<String>>(mut self, name: T) -> Self {
        self.http_requests_total_name = name.into();
        self
    }

    /// Set name for `http_requests_duration_seconds` metric
    pub fn http_requests_duration_seconds_name<T: Into<String>>(mut self, name: T) -> Self {
        self.http_requests_duration_seconds_name = name.into();
        self
    }
}

/// Static references to variable metrics/label names.
/// This config primarily exists to avoid allocations during execution.
#[derive(Debug, Clone)]
struct MetricsMetadata {
    http_requests_total: &'static str,
    http_requests_duration_seconds: &'static str,
    endpoint: &'static str,
    method: &'static str,
    status: &'static str,
    version: Option<&'static str>,
    const_labels: Vec<(&'static str, String)>,
}

/// By default two metrics are tracked:
///
///   - `http_requests_total` (labels: endpoint, method, status): the total
///     number of HTTP requests handled by the actix `HttpServer`.
///
///   - `http_requests_duration_seconds` (labels: endpoint, method, status):
///      the request duration for all HTTP requests handled by the actix `HttpServer`.
#[derive(Clone)]
#[must_use = "must be set up as middleware for actix-web"]
pub struct ActixWebMetrics {
    pub(crate) names: MetricsMetadata,

    pub(crate) exclude: HashSet<String>,
    pub(crate) exclude_regex: RegexSet,
    pub(crate) exclude_status: HashSet<StatusCode>,
    pub(crate) enable_http_version_label: bool,
    pub(crate) unmatched_patterns_mask: Option<String>,
}

impl ActixWebMetrics {
    fn update_metrics(
        &self,
        http_version: Version,
        mixed_pattern: &str,
        fallback_pattern: &str,
        method: &Method,
        status: StatusCode,
        clock: Instant,
        was_path_matched: bool,
    ) {
        if self.exclude.contains(mixed_pattern)
            || self.exclude_regex.is_match(mixed_pattern)
            || self.exclude_status.contains(&status)
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
        } else if let Some(mask) = &self.unmatched_patterns_mask {
            mask
        } else {
            final_pattern
        };

        let mut labels = Vec::with_capacity(4 + self.names.const_labels.len());
        labels.push((self.names.endpoint, final_pattern.to_string()));
        labels.push((self.names.method, method.as_str().to_string()));
        labels.push((self.names.status, status.as_str().to_string()));

        if self.enable_http_version_label {
            labels.push((
                self.names.version.unwrap(),
                Self::http_version_label(http_version).to_string(),
            ));
        }

        for (k, v) in &self.names.const_labels {
            labels.push((k, v.clone()));
        }

        let elapsed = clock.elapsed();
        let duration =
            (elapsed.as_secs() as f64) + f64::from(elapsed.subsec_nanos()) / 1_000_000_000_f64;
        histogram!(self.names.http_requests_duration_seconds, &labels).record(duration);

        counter!(self.names.http_requests_total, &labels).increment(1);
    }

    fn http_version_label(version: Version) -> &'static str {
        match version {
            v if v == Version::HTTP_09 => "HTTP/0.9",
            v if v == Version::HTTP_10 => "HTTP/1.0",
            v if v == Version::HTTP_11 => "HTTP/1.1",
            v if v == Version::HTTP_2 => "HTTP/2.0",
            v if v == Version::HTTP_3 => "HTTP/3.0",
            _ => "<unrecognized>",
        }
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
            inner: Arc::new(self.clone()),
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
        inner: Arc<ActixWebMetrics>,
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
        let params_keep_path_cardinality = match req.extensions_mut().get::<MetricsConfig>() {
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

        let inner = this.inner.clone();
        Poll::Ready(Ok(res.map_body(move |head, body| StreamLog {
            body,
            size: 0,
            clock: time,
            inner,
            status: head.status,
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
    inner: Arc<ActixWebMetrics>,
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
        size: usize,
        clock: Instant,
        inner: Arc<ActixWebMetrics>,
        status: StatusCode,
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
                .update_metrics(this.version, &this.mixed_pattern, &this.fallback_pattern, &this.method, this.status, this.clock, this.was_path_matched);
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
                *this.size += chunk.len();
                Poll::Ready(Some(Ok(chunk)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}
