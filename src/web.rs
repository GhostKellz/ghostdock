use axum::{
    routing::get,
    Router,
    response::Html,
};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/dashboard", get(dashboard))
        .route("/repositories", get(repositories))
        .route("/users", get(users))
        .route("/settings", get(settings))
}

async fn index() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GhostDock Registry</title>
    <style>
        body { font-family: Arial, sans-serif; background: #0f172a; color: #e2e8f0; padding: 2rem; }
        .container { max-width: 1200px; margin: 0 auto; text-align: center; }
        .hero { margin: 4rem 0; }
        .btn { background: #2563eb; color: white; padding: 1rem 2rem; text-decoration: none; border-radius: 8px; margin: 0.5rem; }
    </style>
</head>
<body>
    <div class="container">
        <div class="hero">
            <h1>🐳 GhostDock Registry</h1>
            <p>Next-generation Docker registry with modern UI and enterprise features</p>
            <a href="/dashboard" class="btn">Go to Dashboard</a>
        </div>
    </div>
</body>
</html>"#;
    Html(html.to_string())
}

async fn dashboard() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GhostDock Registry - Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; background: #0f172a; color: #e2e8f0; padding: 2rem; }
        .container { max-width: 1200px; margin: 0 auto; }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem; margin: 2rem 0; }
        .stat-card { background: rgba(30, 41, 59, 0.5); padding: 1.5rem; border-radius: 8px; }
        .btn { background: #2563eb; color: white; padding: 0.5rem 1rem; text-decoration: none; border-radius: 4px; margin: 0.5rem; }
    </style>
</head>
<body>
    <div class="container">
        <h1>GhostDock Dashboard</h1>
        <div class="stats">
            <div class="stat-card"><h3>Repositories</h3><p>12</p></div>
            <div class="stat-card"><h3>Images</h3><p>89</p></div>
            <div class="stat-card"><h3>Storage</h3><p>2.4GB</p></div>
        </div>
        <div>
            <a href="/repositories" class="btn">Browse Registry</a>
            <a href="/settings" class="btn">Settings</a>
        </div>
    </div>
</body>
</html>"#;
    Html(html.to_string())
}

async fn repositories() -> Html<String> {
    Html("<h1>Repositories</h1><p>Docker repositories will be listed here.</p>".to_string())
}

async fn users() -> Html<String> {
    Html("<h1>Users</h1><p>User management interface.</p>".to_string())
}

async fn settings() -> Html<String> {
    Html("<h1>Settings</h1><p>Registry configuration settings.</p>".to_string())
}
