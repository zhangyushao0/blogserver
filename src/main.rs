mod blog;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/api", get(root))
        .route("/api/blogs", get(get_all_post_mata))
        .route("/api/blog", post(get_post_by_link));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_all_post_mata() -> (StatusCode, Json<Vec<blog::Metadata>>) {
    let metas = blog::get_all_post_mata().await.unwrap();
    (StatusCode::OK, Json(metas))
}

async fn get_post_by_link(Json(payload): Json<GetPost>) -> (StatusCode, Json<Post>) {
    let path = blog::covert_link_to_path(&payload.link).await.unwrap();
    let content = blog::get_post_by_path(&path).await.unwrap();
    let meta = blog::get_post_mata_by_path(&path).await.unwrap();
    (
        StatusCode::OK,
        Json(Post {
            date: meta.date,
            content,
        }),
    )
}

#[derive(Deserialize)]
struct GetPost {
    link: String,
}

#[derive(Serialize)]
struct Post {
    date: String,
    content: String,
}
