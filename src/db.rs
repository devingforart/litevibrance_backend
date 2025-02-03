// /home/southatoms/Escritorio/lite_vibrance_web/server/backend_vibrance/src/db.rs
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::env;
use serde_json;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db() -> anyhow::Result<DbPool> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL no se encuentra en .env");

    // Opción para forzar el borrado de la tabla products si existe, descomentando el bloque indicado abajo:
    // Esto garantiza que la tabla se cree nuevamente con la columna 'photos'.
    // Si no deseas recrear la tabla en cada ejecución, gestiona migraciones o elimina manualmente la tabla:
    //
    // let pool_temp = SqlitePoolOptions::new()
    //     .max_connections(1)
    //     .connect(&database_url)
    //     .await?;
    // sqlx::query("DROP TABLE IF EXISTS products")
    //     .execute(&pool_temp)
    //     .await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uuid TEXT NOT NULL,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            price REAL NOT NULL,
            photos TEXT,
            description TEXT
        );
        CREATE TABLE IF NOT EXISTS cart_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_uuid TEXT NOT NULL,
            product_uuid TEXT NOT NULL,
            quantity INTEGER NOT NULL
        );
        "#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn seed_db(pool: &DbPool) -> anyhow::Result<()> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await?;
    
    if count.0 == 0 {
        println!("Sembrando la base de datos con productos iniciales...");

        let products = vec![
            (
                "prod-uuid-1",
                "Cámara Reflex Profesional",
                "camara-reflex-profesional",
                799.99,
                vec![
                    "https://picsum.photos/seed/camera1/600/400",
                    "https://picsum.photos/seed/camera2/600/400",
                    "https://picsum.photos/seed/camera3/600/400",
                    "https://picsum.photos/seed/camera4/600/400",
                ],
                "Esta cámara reflex profesional cuenta con un sensor de alta resolución, rápida velocidad de disparo y ópticas intercambiables para capturar imágenes de alta calidad en cualquier situación. Ideal para fotógrafos profesionales y entusiastas avanzados."
            ),
            (
                "prod-uuid-2",
                "Laptop Ultrabook",
                "laptop-ultrabook",
                999.99,
                vec![
                    "https://images.unsplash.com/photo-1481277542470-605612bd2d61?auto=format&fit=crop&w=600&q=80",
                ],
                "Una laptop elegante y potente que combina portabilidad y rendimiento para profesionales en movimiento."
            ),
            (
                "prod-uuid-3",
                "Smartphone Pro",
                "smartphone-pro",
                899.99,
                vec![
                    "https://images.unsplash.com/photo-1495433324511-bf8e92934d90?auto=format&fit=crop&w=600&q=80",
                ],
                "El Smartphone Pro ofrece una experiencia premium con una pantalla de alta resolución y cámaras avanzadas."
            ),
            (
                "prod-uuid-4",
                "Tablet 10\"",
                "tablet-10",
                349.99,
                vec![
                    "https://images.unsplash.com/photo-1542751371-adc38448a05e?auto=format&fit=crop&w=600&q=80",
                ],
                "Una tablet versátil perfecta para entretenimiento, lectura y productividad."
            ),
        ];

        for (uuid, name, slug, price, photos, description) in products {
            let photos_json = serde_json::to_string(&photos).unwrap();
            sqlx::query(
                "INSERT INTO products (uuid, name, slug, price, photos, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )
            .bind(uuid)
            .bind(name)
            .bind(slug)
            .bind(price)
            .bind(photos_json)
            .bind(description)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}
