use std::fmt::Write;
use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use log::warn;
use rustcommon_metrics::{Counter, Gauge};

async fn index_page() -> Html<String> {
    let body = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8" />
            <title>Augur {version}</title>
            <style>
                body {{
                    font-family: Arial, Helvetica, sans-serif;
                }}
            </style>
        </head>

        <body>
            <h1>Augur {version}</h1>
            <p>Human-readable metrics under <a href="/metrics">/metrics</a>.</p>
            <p>
                Machine-readable metrics under
                <a href="/vars.json">/vars.json</a> or
                <a href="/metrics.json">/metrics.json</a>.
            </p>
        </body>
        </html>
        "#,
        version = env!("CARGO_PKG_VERSION")
    );

    Html(body)
}

async fn json_metrics() -> impl IntoResponse {
    let mut output = String::with_capacity(1024);
    output.push('{');
    let mut separator = "";

    for metric in &rustcommon_metrics::metrics() {
        let any = match metric.as_any() {
            Some(any) => any,
            None => continue,
        };

        if let Some(counter) = any.downcast_ref::<Counter>() {
            let _ = write!(
                output,
                r#"{}"{}":{}"#,
                separator,
                metric.name(),
                counter.value()
            );
        } else if let Some(gauge) = any.downcast_ref::<Gauge>() {
            let _ = write!(
                output,
                r#"{}"{}":{}"#,
                separator,
                metric.name(),
                gauge.value()
            );
        } else {
            warn!("Found metric with unknown type: '{}'", metric.name());
            continue;
        }

        separator = ",";
    }

    output.push('}');

    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        output,
    )
}

async fn text_metrics() -> String {
    let mut output = String::with_capacity(1024);

    for metric in &rustcommon_metrics::metrics() {
        let any = match metric.as_any() {
            Some(any) => any,
            None => continue,
        };

        if let Some(counter) = any.downcast_ref::<Counter>() {
            let _ = writeln!(output, "{}: {}", metric.name(), counter.value());
        } else if let Some(gauge) = any.downcast_ref::<Gauge>() {
            let _ = writeln!(output, "{}: {}", metric.name(), gauge.value());
        } else {
            warn!("Found metric with unknown type: '{}'", metric.name());
        }
    }

    output
}

pub async fn serve_admin(address: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(index_page))
        .route("/metrics", get(text_metrics))
        .route("/metrics.json", get(json_metrics))
        .route("/vars.json", get(json_metrics))
        .route("/admin/metrics.json", get(json_metrics));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .map_err(From::from)
}
