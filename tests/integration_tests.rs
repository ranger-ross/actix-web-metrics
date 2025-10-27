use std::collections::HashMap;

use actix_web::dev::Service;
use actix_web::http::{StatusCode, Version};
use actix_web::test::{call_service, init_service, read_body, TestRequest};
use actix_web::{web, App, HttpMessage, HttpResponse, Resource, Scope};
use actix_web_metrics::{
    ActixWebMetricsBuilder, ActixWebMetricsConfig, ActixWebMetricsExtension, LabelsConfig,
};
use metrics::{counter, set_default_local_recorder, Key, Label};
use metrics_util::debugging::{DebugValue, DebuggingRecorder};
use metrics_util::{CompositeKey, MetricKind};

const SNAPSHOT_FILTERS: [(&str, &str); 2] =
    [(r"\d\.\d+e-\d+", "[VALUE]"), (r"\d\.\d{5, 20}", "[VALUE]")];

#[actix_web::test]
async fn middleware_basic() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new().build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok)),
    )
    .await;

    let res = call_service(&app, TestRequest::with_uri("/health_check").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");
    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_http_version() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new()
        .metrics_config(
            ActixWebMetricsConfig::default().labels(LabelsConfig::default().version("version")),
        )
        .build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok)),
    )
    .await;

    let test_cases = HashMap::from([
        (Version::HTTP_09, 1),
        (Version::HTTP_10, 2),
        (Version::HTTP_11, 5),
        (Version::HTTP_2, 7),
        (Version::HTTP_3, 11),
    ]);

    for (http_version, repeats) in test_cases.iter() {
        for _ in 0..*repeats {
            let res = call_service(
                &app,
                TestRequest::with_uri("/health_check")
                    .version(*http_version)
                    .to_request(),
            )
            .await;
            assert!(res.status().is_success());
            assert_eq!(read_body(res).await, "");
        }
    }

    #[allow(clippy::mutable_key_type)]
    let snap = snapshotter.snapshot().into_hashmap();

    for (http_version, repeats) in test_cases {
        let Some((_, _, DebugValue::Counter(value))) = snap.get(&CompositeKey::new(
            MetricKind::Counter,
            Key::from_name("http_requests_total").with_extra_labels(vec![
                Label::new("endpoint", "/health_check"),
                Label::new("method", "GET"),
                Label::new("status", "200"),
                Label::new("version", format!("{http_version:?}")),
            ]),
        )) else {
            panic!("Missing metric for {http_version:?}");
        };

        assert_eq!(value, &repeats);
    }
}

#[actix_web::test]
async fn middleware_match_pattern() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new().build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/resource/{id}").to(HttpResponse::Ok)),
    )
    .await;

    let res = call_service(&app, TestRequest::with_uri("/resource/123").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_with_mask_unmatched_pattern() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new()
        .mask_unmatched_patterns("UNKNOWN")
        .build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/resource/{id}").to(HttpResponse::Ok)),
    )
    .await;

    let res = call_service(&app, TestRequest::with_uri("/not-real").to_request()).await;
    assert!(res.status().is_client_error());
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_with_mixed_params_cardinality() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    // we want to keep metrics label on the "cheap param" but not on the "expensive" param
    let prometheus = ActixWebMetricsBuilder::new().build();

    let app = init_service(
        App::new().wrap(prometheus).service(
            web::resource("/resource/{cheap}/{expensive}")
                .wrap_fn(|req, srv| {
                    req.extensions_mut().insert::<ActixWebMetricsExtension>(
                        ActixWebMetricsExtension {
                            cardinality_keep_params: vec!["cheap".to_string()],
                        },
                    );
                    srv.call(req)
                })
                .to(|path: web::Path<(String, String)>| async {
                    let (cheap, _expensive) = path.into_inner();
                    if !["foo", "bar"].map(|x| x.to_string()).contains(&cheap) {
                        return HttpResponse::NotFound().finish();
                    }
                    HttpResponse::Ok().finish()
                }),
        ),
    )
    .await;

    // first probe to check basic facts
    let res = call_service(
        &app,
        TestRequest::with_uri("/resource/foo/12345").to_request(),
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });

    // second probe to test 404 behavior
    let res = call_service(
        &app,
        TestRequest::with_uri("/resource/invalid/92945").to_request(),
    )
    .await;
    assert!(res.status() == 404);
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_basic_failure() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new()
        .disable_unmatched_pattern_masking()
        .build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok)),
    )
    .await;

    call_service(&app, TestRequest::with_uri("/health_checkz").to_request()).await;

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_custom_counter() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new().build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok)),
    )
    .await;

    // Verify that 'counter' does not appear in the output before we use it
    call_service(&app, TestRequest::with_uri("/health_check").to_request()).await;

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });

    counter!("counter").increment(1);

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_const_labels() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let mut labels = HashMap::new();
    labels.insert("label1".to_string(), "value1".to_string());
    labels.insert("label2".to_string(), "value2".to_string());
    let prometheus = ActixWebMetricsBuilder::new().const_labels(labels).build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok)),
    )
    .await;

    let res = call_service(&app, TestRequest::with_uri("/health_check").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_metrics_config() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let metrics_config = ActixWebMetricsConfig::default()
        .http_requests_duration_seconds_name("my_http_request_duration")
        .http_requests_total_name("my_http_requests_total");

    let prometheus = ActixWebMetricsBuilder::new()
        .metrics_config(metrics_config)
        .build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok)),
    )
    .await;

    let res = call_service(&app, TestRequest::with_uri("/health_check").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[test]
fn compat_with_non_boxed_middleware() {
    let _app = App::new()
        .wrap(ActixWebMetricsBuilder::new().build())
        .wrap(actix_web::middleware::Logger::default())
        .route("", web::to(|| async { "" }));

    let _app = App::new()
        .wrap(actix_web::middleware::Logger::default())
        .wrap(ActixWebMetricsBuilder::new().build())
        .route("", web::to(|| async { "" }));

    let _scope = Scope::new("")
        .wrap(ActixWebMetricsBuilder::new().build())
        .route("", web::to(|| async { "" }));

    let _resource = Resource::new("")
        .wrap(ActixWebMetricsBuilder::new().build())
        .route(web::to(|| async { "" }));
}

#[actix_web::test]
async fn middleware_excludes() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new()
        .exclude("/ping")
        .exclude_regex("/readyz/.*")
        .exclude_status(StatusCode::NOT_FOUND)
        .build();

    let app = init_service(
        App::new()
            .wrap(prometheus)
            .service(web::resource("/health_check").to(HttpResponse::Ok))
            .service(web::resource("/ping").to(HttpResponse::Ok))
            .service(web::resource("/readyz/{subsystem}").to(HttpResponse::Ok)),
    )
    .await;

    let res = call_service(&app, TestRequest::with_uri("/health_check").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let res = call_service(&app, TestRequest::with_uri("/ping").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let res = call_service(&app, TestRequest::with_uri("/readyz/database").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "");

    let res = call_service(&app, TestRequest::with_uri("/notfound").to_request()).await;
    assert!(res.status().is_client_error());
    assert_eq!(read_body(res).await, "");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}

#[actix_web::test]
async fn middleware_with_size_metrics() {
    let recorder = DebuggingRecorder::new();
    let snapshotter = recorder.snapshotter();
    let _guard = set_default_local_recorder(&recorder);

    let prometheus = ActixWebMetricsBuilder::new().build();

    let app = init_service(App::new().wrap(prometheus).service(
        web::resource("/health_check").to(|| async { HttpResponse::Ok().body("test response") }),
    ))
    .await;

    let res = call_service(&app, TestRequest::with_uri("/health_check").to_request()).await;
    assert!(res.status().is_success());
    assert_eq!(read_body(res).await, "test response");

    let snapshot = snapshotter.snapshot();
    insta::with_settings!({filters => SNAPSHOT_FILTERS}, {
        insta::assert_debug_snapshot!(snapshot);
    });
}
