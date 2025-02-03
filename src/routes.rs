// ... (el resto del archivo permanece igual)

use axum::{
    extract::{State, Path, TypedHeader},
    http::{StatusCode, Request},
    headers::{Authorization, authorization::Bearer},
    Json,
};
use sqlx::{Row, Pool, Sqlite};
use crate::models::{ Product, AddToCartRequest, CartItem };
use crate::auth::decode_jwt;

pub type DbPool = Pool<Sqlite>;

// -------------------- 1) Endpoint de salud --------------------
pub async fn health_check() -> &'static str {
    "OK"
}

// -------------------- 2) Obtener lista de productos --------------------
pub async fn get_products(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<Product>>, (StatusCode, String)> {
    let query = r#"SELECT uuid, name, slug, price, photos, description FROM products"#;
    let rows = sqlx::query_as::<_, Product>(query)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

// -------------------- 3) Obtener producto por ID --------------------
pub async fn get_product_by_id(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> Result<Json<Product>, (StatusCode, String)> {
    let query = r#"
        SELECT uuid, name, slug, price, photos, description
        FROM products
        WHERE slug = ?1 OR uuid = ?1
    "#;

    let product = sqlx::query_as::<_, Product>(query)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error al buscar producto: {}", e)))?;

    if let Some(p) = product {
        Ok(Json(p))
    } else {
        Err((StatusCode::NOT_FOUND, "Producto no encontrado".to_string()))
    }
}

// -------------------- 4) Agregar items al carrito (requiere token) --------------------
pub async fn add_to_cart(
    State(pool): State<DbPool>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<AddToCartRequest>,
) -> Result<String, (StatusCode, String)> {
    let token = bearer.token();
    let claims = decode_jwt(token)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Token inválido".to_string()))?;
    let user_uuid = claims.sub;

    let query = r#"
        INSERT INTO cart_items (user_uuid, product_uuid, quantity)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO NOTHING;
    "#;
    sqlx::query(query)
        .bind(user_uuid)
        .bind(payload.product_uuid)
        .bind(payload.quantity)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok("Producto agregado al carrito".to_string())
}

// -------------------- 5) Ver carrito --------------------
pub async fn get_cart(
    State(pool): State<DbPool>,
    req: Request<axum::body::Body>,
) -> Result<Json<Vec<CartItem>>, (StatusCode, String)> {
    let user_uuid = get_user_uuid_from_header(&req).await?;

    let query = r#"
        SELECT 
            ci.product_uuid, 
            p.name, 
            p.price, 
            json_extract(p.photos, '$[0]') as image,
            ci.quantity 
        FROM cart_items ci
        JOIN products p ON ci.product_uuid = p.uuid
        WHERE ci.user_uuid = ?1
    "#;
    let cart_items = sqlx::query_as::<_, CartItem>(query)
        .bind(user_uuid)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(cart_items))
}

// -------------------- 6) Eliminar producto del carrito --------------------
pub async fn remove_from_cart(
    State(pool): State<DbPool>,
    Path(product_id): Path<String>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<String, (StatusCode, String)> {
    let token = bearer.token();
    let claims = decode_jwt(token)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Token inválido".to_string()))?;
    let user_uuid = claims.sub;

    let query = r#"
        DELETE FROM cart_items
        WHERE user_uuid = ?1 AND product_uuid = ?2
    "#;
    sqlx::query(query)
        .bind(user_uuid)
        .bind(product_id.clone())
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(format!("Producto '{}' eliminado del carrito", product_id))
}

// -------------------- 7) Checkout --------------------
pub async fn checkout(
    State(pool): State<DbPool>,
    req: Request<axum::body::Body>,
) -> Result<String, (StatusCode, String)> {
    let user_uuid = get_user_uuid_from_header(&req).await?;

    let query = r#"DELETE FROM cart_items WHERE user_uuid = ?1"#;
    sqlx::query(query)
        .bind(user_uuid)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok("Compra finalizada con éxito!".to_string())
}

// -------------------- Función auxiliar para extraer el usuario del header --------------------
async fn get_user_uuid_from_header(
    req: &Request<axum::body::Body>,
) -> Result<String, (StatusCode, String)> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or((StatusCode::UNAUTHORIZED, "Falta header Authorization".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Header inválido".to_string()))?;

    if !auth_str.starts_with("Bearer ") {
        return Err((StatusCode::BAD_REQUEST, "Token debe ser Bearer".to_string()));
    }

    let token = auth_str.trim_start_matches("Bearer ");
    let claims = decode_jwt(token)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Token inválido".to_string()))?;

    Ok(claims.sub)
}
