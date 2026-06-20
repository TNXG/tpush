use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct MessageRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub payload: String,
    pub kind: String,
    pub received_at: String,
    pub read: bool,
}

#[derive(Clone)]
pub struct MessageStore {
    connection: Arc<Mutex<Connection>>,
}

impl MessageStore {
    pub fn open(database_path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(database_path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                payload TEXT NOT NULL,
                kind TEXT NOT NULL,
                received_at TEXT NOT NULL,
                read INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS deleted_messages (
                id TEXT PRIMARY KEY,
                deleted_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn insert_message(&self, message: &MessageRecord) -> Result<()> {
        let connection = self.connection.lock().unwrap();
        if Self::is_message_hidden(&connection, message)? {
            return Ok(());
        }

        connection.execute(
            r#"
            INSERT OR IGNORE INTO messages (id, title, content, payload, kind, received_at, read)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                message.id,
                message.title,
                message.content,
                message.payload,
                message.kind,
                message.received_at,
                if message.read { 1_i64 } else { 0_i64 }
            ],
        )?;
        Ok(())
    }

    pub fn get_messages(&self) -> Result<Vec<MessageRecord>> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection.prepare(
            r#"
            SELECT id, title, content, payload, kind, received_at, read
            FROM messages
            ORDER BY received_at DESC
            "#,
        )?;
        let rows = statement.query_map([], |row| {
            Ok(MessageRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                payload: row.get(3)?,
                kind: row.get(4)?,
                received_at: row.get(5)?,
                read: row.get::<_, i64>(6)? != 0,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn mark_read(&self, id: &str) -> Result<()> {
        let connection = self.connection.lock().unwrap();
        connection.execute("UPDATE messages SET read = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_message(&self, id: &str) -> Result<()> {
        let connection = self.connection.lock().unwrap();
        Self::record_deleted_message(&connection, id)?;
        connection.execute("DELETE FROM messages WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        let connection = self.connection.lock().unwrap();
        let latest_received_at = connection
            .query_row(
                "SELECT COALESCE(MAX(received_at), '') FROM messages",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_default();
        let clear_all_before = if latest_received_at.is_empty() {
            Utc::now().to_rfc3339()
        } else {
            latest_received_at
        };
        Self::set_setting_inner(&connection, "clear_all_before", &clear_all_before)?;
        connection.execute("DELETE FROM messages", [])?;
        Ok(())
    }

    pub fn get_or_create_device_id(&self) -> Result<String> {
        if let Ok(device_id) = self.get_setting("device_id") {
            if !device_id.is_empty() {
                return Ok(device_id);
            }
        }

        let device_id = uuid::Uuid::new_v4().to_string();
        self.set_setting("device_id", &device_id)?;
        Ok(device_id)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let connection = self.connection.lock().unwrap();
        Self::set_setting_inner(&connection, key, value)
    }

    fn set_setting_inner(connection: &Connection, key: &str, value: &str) -> Result<()> {
        connection.execute(
            r#"
            INSERT INTO settings (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![key, value],
        )?;
        Ok(())
    }

    fn get_setting_inner(connection: &Connection, key: &str) -> Result<String> {
        let value = connection.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        Ok(value.unwrap_or_default())
    }

    fn get_setting(&self, key: &str) -> Result<String> {
        let connection = self.connection.lock().unwrap();
        let value = connection.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        Ok(value.unwrap_or_default())
    }

    fn record_deleted_message(connection: &Connection, id: &str) -> Result<()> {
        connection.execute(
            r#"
            INSERT INTO deleted_messages (id, deleted_at)
            VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            ON CONFLICT(id) DO UPDATE SET deleted_at = excluded.deleted_at
            "#,
            params![id],
        )?;
        Ok(())
    }

    fn is_message_hidden(connection: &Connection, message: &MessageRecord) -> Result<bool> {
        let deleted_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM deleted_messages WHERE id = ?1",
            params![message.id],
            |row| row.get(0),
        )?;
        if deleted_count > 0 {
            return Ok(true);
        }

        let clear_all_before = Self::get_setting_inner(connection, "clear_all_before")?;
        Ok(!clear_all_before.is_empty() && message.received_at <= clear_all_before)
    }
}
