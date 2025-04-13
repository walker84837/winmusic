use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub type DbPool = Pool<Postgres>;

pub async fn init_db_pool(db_url: &str, pool_size: u32) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(db_url)
        .await
}
