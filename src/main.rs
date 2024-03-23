mod blog;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service, post},
    Json, Router,
};
use blog::Metadata;
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/api", get(root))
        .route("/api/blogs", get(get_all_post_meta))
        .route("/api/blog", post(get_post_by_link))
        .nest_service("/api/blog/image", get_service(ServeDir::new("blog/image")));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_all_post_meta() -> Result<Json<Vec<Metadata>>, MyError> {
    let metas = blog::get_all_post_meta().await?;
    Ok(Json(metas))
}

async fn get_post_by_link(Json(payload): Json<GetPost>) -> Result<Json<Post>, MyError> {
    let path = blog::covert_link_to_path(&payload.link).await?;
    let meta = blog::get_post_mata_by_path(&path).await?;
    let content = blog::get_post_by_path(&path).await?;
    Ok(Json(Post {
        date: meta.date,
        content,
    }))
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

// 自定义错误类型
struct MyError(Box<dyn std::error::Error>);

impl From<Box<dyn std::error::Error>> for MyError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        MyError(err)
    }
}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}
