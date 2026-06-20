use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub username: String,
    pub password: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| server_project_dir().join("config.toml"));
        if !path.exists() {
            let config = Self::default();
            config.write_default_file(&path)?;
            tracing::warn!(path = %path.display(), "config file missing; created default config");
            return Ok(config.with_env_overrides());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(config.with_env_overrides())
    }

    fn write_default_file(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config directory {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self).context("failed to serialize default config")?;
        std::fs::write(path, content)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    fn with_env_overrides(mut self) -> Self {
        if let Ok(bind_address) = std::env::var("BIND_ADDRESS") {
            self.server.bind_address = bind_address;
        }
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            self.auth.jwt_secret = jwt_secret;
        }
        if let Ok(username) = std::env::var("ADMIN_USERNAME") {
            self.auth.username = username;
        }
        if let Ok(password) = std::env::var("ADMIN_PASSWORD") {
            self.auth.password = password;
        }
        self
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind_address: "0.0.0.0:3000".to_owned(),
            },
            auth: AuthConfig {
                jwt_secret: "change-me-in-production".to_owned(),
                username: "admin".to_owned(),
                password: "admin".to_owned(),
            },
        }
    }
}

pub fn server_project_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
