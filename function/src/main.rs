use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use concert_vitae_function::add;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(show_index))
        .route("/add", post(add));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn show_index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <title>Function</title>
  </head>
  <body>
    <h1>Concert added</h1>
      <p>Head over to <a href="/">the frontpage</a> to see the new concert.</p>
  </body>
</html>
"#,
    )
}
