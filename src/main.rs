use axum::{
    extract::{Extension, Path},
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    routing::post,
    Json, Router, Server, TypedHeader,
};
use axum_auth::AuthBearer;
use headers::{Authorization, ContentType, Expires};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::{
    io::{self, Write},
    time::{Duration, SystemTime},
};
use templates::statics::StaticFile;
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;
use tokio::sync::Mutex;

#[macro_use]
mod axum_ructe;

struct AppState {
    beeps: Mutex<Vec<Beep>>,
}

/// Setup routes
fn app() -> Router {
    let my_beeps = Mutex::new(Vec::new());
    let app_state = Arc::new(AppState { beeps: my_beeps });

    Router::new()
        .route("/", get(home_page))
        .route("/beeps", post(post_beep))
        .route("/static/:filename", get(static_files))
        //        .route("/int/:n", get(take_int))
        .route("/bad", get(make_error))
        .fallback(handler_404.into_service())
        .layer(Extension(app_state))
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

async fn post_beep(
    Extension(state): Extension<Arc<AppState>>,
    AuthBearer(token): AuthBearer,
    Json(input): Json<CreateTodo>,
) -> impl IntoResponse {
    let foo = env::var("MADIK").unwrap();

    format!("Found a bearer token: {}", token);
    // compare bearer token
    if token != foo {
        return StatusCode::NOT_FOUND;
    }

    println!("Got a beep: {}", input.text);

    let mut beeps = state.beeps.lock().await;

    beeps.push(Beep {
        text: input.text,
        timestamp: OffsetDateTime::now_utc().format(&Rfc2822).unwrap(),
    });

    StatusCode::CREATED
}

#[derive(Debug, Deserialize, Clone)]
pub struct Beep {
    text: String,
    timestamp: String,
}

/// Home page handler; just render a template with some arguments.
async fn home_page(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let beeps = state.beeps.lock().await;

    let mut demo = beeps.clone();
    demo.reverse();
    render!(templates::page, &demo)
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
async fn static_files(Path(filename): Path<String>) -> impl IntoResponse {
    /// A duration to add to current time for a far expires header.
    static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);
    match StaticFile::get(&filename) {
        Some(data) => {
            let far_expires = SystemTime::now() + FAR;
            (
                TypedHeader(ContentType::from(data.mime.clone())),
                TypedHeader(Expires::from(far_expires)),
                data.content,
            )
                .into_response()
        }
        None => handler_404().await.into_response(),
    }
}

async fn take_int(payload: Option<Path<usize>>) -> Response {
    if let Some(Path(n)) = payload {
        render!(templates::page, &[]).into_response()
    } else {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sorry, Something went wrong. This is probably not your fault.",
        )
        .into_response()
    }
}

async fn make_error() -> Result<impl IntoResponse, ExampleAppError> {
    let i: i8 = "three".parse()?;
    Ok(render!(templates::page, &[]))
}

/// The error type that can be returned from resource handlers.
///
/// This needs to be convertible from any error types used with `?` in
/// handlers, and implement the actix ResponseError type.
#[derive(Debug)]
enum ExampleAppError {
    ParseInt(std::num::ParseIntError),
}
impl std::fmt::Display for ExampleAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ExampleAppError {}

impl From<std::num::ParseIntError> for ExampleAppError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}
impl IntoResponse for ExampleAppError {
    fn into_response(self) -> Response {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sorry, Something went wrong. This is probably not your fault.",
        )
        .into_response()
    }
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut impl Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("love", "https://crates.io/crates/axum"),
            ("tears", "https://crates.io/crates/ructe"),
        ],
    )
}

async fn handler_404() -> impl IntoResponse {
    error_response(
        StatusCode::NOT_FOUND,
        "The resource you requested can't be found.",
    )
}

fn error_response(status_code: StatusCode, message: &str) -> impl IntoResponse + '_ {
    (status_code, render!(templates::error, status_code, message))
}

/// Start server
#[tokio::main]
async fn main() {
    env_logger::init();
    Server::bind(&"0.0.0.0:7331".parse().unwrap())
        .serve(app().into_make_service())
        .await
        .unwrap()
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
