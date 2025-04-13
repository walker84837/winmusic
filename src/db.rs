use crate::SpotifyTrack;
use log::{error, info};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(db_url: &str, pool_size: u32) -> Result<Self, sqlx::Error> {
        info!("Initializing database pool with size {}", pool_size);
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(db_url)
            .await?;
        info!("Database pool initialized successfully");
        Ok(Self { pool })
    }

    pub async fn add_playlist_tracks(
        &self,
        playlist_id: &str,
        tracks: &[SpotifyTrack],
    ) -> Result<(), sqlx::Error> {
        info!(
            "Adding {} tracks from playlist {}",
            tracks.len(),
            playlist_id
        );
        let mut tx = self.pool.begin().await?;

        for track in tracks {
            match sqlx::query!(
                "INSERT INTO tracks (name, artist, playlist_id) VALUES ($1, $2, $3)",
                track.name,
                track.artist,
                playlist_id
            )
            .execute(&mut *tx)
            .await
            {
                Ok(_) => info!("Added track: {} - {}", track.name, track.artist),
                Err(e) => {
                    error!("Failed to add track: {} - {}", track.name, track.artist);
                    return Err(e);
                }
            }
        }

        tx.commit().await?;
        info!("Successfully added all tracks");
        Ok(())
    }
}
