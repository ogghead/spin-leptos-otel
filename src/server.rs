mod parsing;

use leptos::{config::get_configuration, error::Error};
use leptos_wasi::{
    handler::HandlerError,
    prelude::{Executor, IncomingRequest, ResponseOutparam, WasiExecutor},
};
use opentelemetry_proto::tonic::{
    collector::{
        logs::v1::ExportLogsServiceRequest, metrics::v1::ExportMetricsServiceRequest,
        trace::v1::ExportTraceServiceRequest,
    },
    common::v1::AnyValue,
};
use parsing::{parse_key_values_to_sorted_string, parse_value_to_str};
use prost::Message;
use wasi::{exports::http::incoming_handler::Guest, http::types::OutgoingResponse};
use wasi::{
    http::{proxy::export, types::Headers},
    io::streams::StreamError,
};

use crate::app::{shell, App, SaveCount};

struct LeptosServer;

impl Guest for LeptosServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        if request
            .path_with_query()
            .map(|path| path.contains("/v1/"))
            .unwrap_or_default()
        {
            if let Err(e) = handle_otel_request(request, response_out) {
                eprintln!("Got error while handling OTel request: {e:?}");
            }
        } else {
            let executor = WasiExecutor::new(leptos_wasi::executor::Mode::Stalled);
            // Initiate a single-threaded [`Future`] Executor so we can run the
            // rendering system and take advantage of bodies streaming.
            if let Err(e) = Executor::init_local_custom_executor(executor.clone()) {
                eprintln!("Got error while initializing leptos_wasi executor: {e:?}");
                return;
            }

            executor.run_until(async {
                if let Err(e) = handle_leptos_request(request, response_out).await {
                    eprintln!("Got error while handling request: {e:?}");
                }
            })
        }
    }
}

fn handle_otel_request(
    request: IncomingRequest,
    response_out: ResponseOutparam,
) -> Result<(), Error> {
    let body = request.consume().expect("couldn't get body");
    let body_stream = body.stream().expect("could not create a stream from body");

    let mut body_bytes = Vec::<u8>::with_capacity(64);

    let wasi_res = OutgoingResponse::new(Headers::new());
    wasi_res
        .set_status_code(200)
        .expect("invalid http status code was returned");

    loop {
        match body_stream.blocking_read(64) {
            Err(StreamError::Closed) => break,
            Err(StreamError::LastOperationFailed(err)) => {
                println!("Got error while parsing body: {err}");
                wasi_res
                    .set_status_code(503)
                    .expect("invalid http status code was returned");
                break;
            }
            Ok(data) => {
                body_bytes.extend(data);
            }
        }
    }

    match request.path_with_query() {
        Some(s) => match s.as_str() {
            "/v1/metrics" => {
                let parsed = ExportMetricsServiceRequest::decode(body_bytes.as_slice())?;
                println!("TODO -- parse metrics: {parsed:?}!");
            }
            "/v1/traces" => {
                let parsed = ExportTraceServiceRequest::decode(body_bytes.as_slice())?;
                println!("TODO -- parse traces: {parsed:?}!");
            }
            "/v1/logs" => {
                let parsed = ExportLogsServiceRequest::decode(body_bytes.as_slice())?;

                let conn = spin_sdk::sqlite::Connection::open_default()?;
                for resource_log in parsed.resource_logs {
                    let resource_labels = resource_log
                        .resource
                        .map(|res| res.attributes)
                        .unwrap_or_default();
                    let resource_labels_str = parse_key_values_to_sorted_string(resource_labels);

                    for scope_log in resource_log.scope_logs {
                        let scope_labels = scope_log
                            .scope
                            .map(|scope| scope.attributes)
                            .unwrap_or_default();
                        let scope_labels_str = parse_key_values_to_sorted_string(scope_labels);

                        for log in scope_log.log_records {
                            let log_labels_str = parse_key_values_to_sorted_string(log.attributes);
                            conn.execute(
                                "INSERT INTO logs (resource_labels, scope_labels, log_labels, log) VALUES (?, ?, ?, ?)",
                                &[
                                    spin_sdk::sqlite::Value::Blob(resource_labels_str.clone().into_bytes()),
                                    spin_sdk::sqlite::Value::Blob(
                                        scope_labels_str.clone().into_bytes(),
                                    ),
                                    spin_sdk::sqlite::Value::Blob(
                                        log_labels_str.into_bytes(),
                                    ),
                                    spin_sdk::sqlite::Value::Text(match log.body {
                                        Some(AnyValue { value: Some(val) }) => 
                                             parse_value_to_str(val),
                                        _ => "".to_string(),
                                    })
                                ],
                            )?;
                        }
                    }
                }
            }
            _ => panic!("TODO: Handle exception cases"),
        },
        None => panic!("TODO: Handle exception cases"),
    };

    ResponseOutparam::set(response_out, Ok(wasi_res));

    Ok(())
}

async fn handle_leptos_request(
    request: IncomingRequest,
    response_out: ResponseOutparam,
) -> Result<(), HandlerError> {
    use leptos_wasi::prelude::Handler;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    Handler::build(request, response_out)?
        // NOTE: Add all server functions here to ensure functionality works as expected!
        .with_server_fn::<SaveCount>()
        // Fetch all available routes from your App.
        .generate_routes(App)
        // Actually process the request and write the response.
        .handle_with_context(move || shell(leptos_options.clone()), || {})
        .await?;
    Ok(())
}

export!(LeptosServer with_types_in wasi);
