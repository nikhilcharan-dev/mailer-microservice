pub mod logs;
pub mod templates;
pub mod transports;
pub mod users;

use anyhow::{Context, Result};
use mongodb::{
    bson::doc,
    options::{ClientOptions, IndexOptions},
    Client, Database, IndexModel,
};
use std::time::Duration;

pub struct Db {
    #[allow(dead_code)] // kept for future admin/migration tooling
    pub client: Client,
    pub db: Database,
}

impl Db {
    pub async fn connect() -> Result<Self> {
        let uri = std::env::var("MONGODB_URI").context("MONGODB_URI required")?;
        let mut opts = ClientOptions::parse(&uri)
            .await
            .context("invalid MONGODB_URI")?;
        opts.max_pool_size = Some(
            std::env::var("MONGODB_MAX_POOL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
        );
        let client = Client::with_options(opts).context("mongo client init failed")?;
        let db_name = std::env::var("MONGODB_DATABASE")
            .unwrap_or_else(|_| "central_mailer".to_string());
        let db = client.database(&db_name);
        Ok(Self { client, db })
    }

    pub async fn ensure_indexes(&self) -> Result<()> {
        let unique = IndexOptions::builder().unique(true).build();

        // users
        let users = self.db.collection::<crate::models::user::UserDoc>("users");
        users
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "username": 1 })
                    .options(unique.clone())
                    .build(),
                None,
            )
            .await?;
        users
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "email": 1 })
                    .options(unique.clone())
                    .build(),
                None,
            )
            .await?;
        users
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "api_key": 1 })
                    .options(unique.clone())
                    .build(),
                None,
            )
            .await?;

        // transports
        let transports = self
            .db
            .collection::<crate::models::transport::TransportDoc>("transports");
        transports
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1, "name": 1 })
                    .options(unique.clone())
                    .build(),
                None,
            )
            .await?;
        transports
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1 })
                    .build(),
                None,
            )
            .await?;

        // templates
        let templates = self
            .db
            .collection::<crate::models::template::TemplateDoc>("templates");
        templates
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1, "name": 1 })
                    .options(unique.clone())
                    .build(),
                None,
            )
            .await?;
        templates
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1 })
                    .build(),
                None,
            )
            .await?;

        // delivery_logs
        let logs = self.db.collection::<crate::models::log::LogDoc>("delivery_logs");
        logs.create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "sent_at": -1 })
                .build(),
            None,
        )
        .await?;
        logs.create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "status": 1 })
                .build(),
            None,
        )
        .await?;
        let retention_days: u64 = std::env::var("LOG_RETENTION_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(90);
        logs.create_index(
            IndexModel::builder()
                .keys(doc! { "sent_at": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(Duration::from_secs(retention_days * 86_400))
                        .build(),
                )
                .build(),
            None,
        )
        .await?;

        Ok(())
    }

    pub async fn ping(&self) -> Result<()> {
        self.db.run_command(doc! { "ping": 1 }, None).await?;
        Ok(())
    }
}
