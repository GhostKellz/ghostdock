use axum::response::Html;

pub async fn index() -> Html<&'static str> {
    Html("Web handler placeholder")
}
