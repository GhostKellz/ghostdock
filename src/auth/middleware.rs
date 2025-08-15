// Auth middleware
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    // TODO: Implement authentication middleware
    next.run(request).await
}
