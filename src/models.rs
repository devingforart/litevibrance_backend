// /home/southatoms/Escritorio/lite_vibrance_web/server/backend_vibrance/src/models.rs
use serde::{Serialize, Deserialize};
use sqlx::{Row, Error as SqlxError};
use sqlx::sqlite::SqliteRow;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub price: f64,
    pub photos: Vec<String>,
    pub description: String,
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for Product {
    fn from_row(row: &SqliteRow) -> Result<Self, SqlxError> {
        let uuid: String = row.try_get("uuid")?;
        let name: String = row.try_get("name")?;
        let slug: String = row.try_get("slug")?;
        let price: f64 = row.try_get("price")?;
        let photos_text: Option<String> = row.try_get("photos")?;
        let description: Option<String> = row.try_get("description")?;

        let parsed_photos = match photos_text {
            Some(txt) => serde_json::from_str::<Vec<String>>(&txt).unwrap_or_default(),
            None => vec![],
        };

        Ok(Product {
            uuid,
            name,
            slug,
            price,
            photos: parsed_photos,
            description: description.unwrap_or_else(|| "".to_string()),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub product_uuid: String,
    pub name: String,
    pub image: String,
    pub price: f64,
    pub quantity: i32,
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for CartItem {
    fn from_row(row: &SqliteRow) -> Result<Self, SqlxError> {
        let product_uuid: String = row.try_get("product_uuid")?;
        let name: String = row.try_get("name")?;
        let price: f64 = row.try_get("price")?;
        let image: String = row.try_get("image")?;
        let quantity: i32 = row.try_get("quantity")?;
        Ok(CartItem { product_uuid, name, price, image, quantity })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToCartRequest {
    pub product_uuid: String,
    pub quantity: i32,
}
