use axum::{
    Router,
    routing::{get, post, delete},
};
use dotenv::dotenv;
use std::net::SocketAddr;

// Importa el middleware de CORS
use tower_http::cors::{CorsLayer, Any};
use axum::http::HeaderValue;

mod db;
mod auth;
mod models;
mod routes;

use routes::{
    health_check,
    get_products, get_product_by_id,
    add_to_cart, get_cart,
    remove_from_cart, checkout,
};

/// Ruta de bienvenida en `/`
async fn root() -> &'static str {
    "Bienvenido a la API de Lite Vibrance"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let pool = db::init_db().await?;
    // Sembrar la base de datos (insertar productos de ejemplo)
    db::seed_db(&pool).await?;

    // Configura CORS para permitir peticiones desde el frontend (http://localhost:5173)
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))  // <- Nueva ruta raíz para evitar 404
        .route("/api/health", get(health_check))
        .route("/api/products", get(get_products))
        .route("/api/products/:id", get(get_product_by_id))
        .route("/api/cart", post(add_to_cart))
        .route("/api/cart", get(get_cart))
        .route("/api/cart/:product_id", delete(remove_from_cart))
        .route("/api/checkout", post(checkout))
        .with_state(pool)
        .layer(cors); // Agrega el middleware de CORS

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse::<u16>()?));

    println!("Servidor corriendo en http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
