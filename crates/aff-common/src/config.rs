use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub admin: DefaultAdminConfig,
    #[serde(default)]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub public_base_url: Option<String>,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultAdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub allow_command_action: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_command_action: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config").required(true))
            .add_source(config::File::with_name("config.local").required(false))
            .add_source(config::Environment::with_prefix("AFF").separator("_"))
            .build()?;

        cfg.try_deserialize()
    }

    pub fn get_public_base_url(&self) -> String {
        self.server
            .public_base_url
            .clone()
            .unwrap_or_else(|| format!("http://{}:{}", self.server.host, self.server.port))
    }

    /// Compute the effective list of allowed CORS origins.
    /// Priority: explicit allowed_origins > public_base_url > local dev default.
    pub fn get_allowed_origins(&self) -> Vec<String> {
        if !self.server.allowed_origins.is_empty() {
            return self.server.allowed_origins.clone();
        }
        let local = format!("http://{}:{}", self.server.host, self.server.port);
        match &self.server.public_base_url {
            Some(url) => vec![url.clone(), local],
            None => vec![local, "http://localhost:5173".to_string()],
        }
    }
}
