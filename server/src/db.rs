use chrono::Utc;
use sqlx::AssertSqlSafe;
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

pub async fn migrate(database: &Pool<Sqlite>) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL UNIQUE,
            channel TEXT NOT NULL DEFAULT 'default',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            key TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS push_messages (
            id TEXT PRIMARY KEY,
            channel TEXT NOT NULL DEFAULT 'default',
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            extras TEXT NOT NULL,
            delivery_status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(database)
    .await?;

    add_column_if_missing(
        database,
        "devices",
        "channel",
        "TEXT NOT NULL DEFAULT 'default'",
    )
    .await?;
    add_column_if_missing(
        database,
        "push_messages",
        "channel",
        "TEXT NOT NULL DEFAULT 'default'",
    )
    .await?;

    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO channels (id, name, key, created_at, updated_at)
        VALUES (?1, 'default', '', ?2, ?3)
        ON CONFLICT(name) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(now)
    .bind(now)
    .execute(database)
    .await?;
    Ok(())
}

async fn add_column_if_missing(
    database: &Pool<Sqlite>,
    table: &str,
    column: &str,
    definition: &str,
) -> anyhow::Result<()> {
    let pragma = format!("PRAGMA table_info({table})");
    let columns = sqlx::query::<Sqlite>(AssertSqlSafe(pragma))
        .fetch_all(database)
        .await?;
    if columns
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .any(|name| name == column)
    {
        return Ok(());
    }

    let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
    sqlx::query::<Sqlite>(AssertSqlSafe(alter))
        .execute(database)
        .await?;
    Ok(())
}
