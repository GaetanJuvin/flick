use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub redis_url: String,
    pub cors_origin: String,
    pub session_secret: String,
    pub auth_mode: AuthMode,
    pub saml: Option<SamlConfig>,
    pub log_level: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthMode {
    Password,
    Saml,
    Both,
}

#[derive(Clone, Debug)]
pub struct SamlConfig {
    pub entry_point: String,
    pub issuer: String,
    pub idp_cert: String,
    pub callback_url: String,
    pub attr_email: String,
    pub attr_name: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let auth_mode = match env::var("AUTH_MODE").unwrap_or_else(|_| "password".into()).as_str() {
            "password" => AuthMode::Password,
            "saml" => AuthMode::Saml,
            "both" => AuthMode::Both,
            other => panic!("Invalid AUTH_MODE: \"{}\". Must be one of: password, saml, both", other),
        };

        let saml = Self::read_saml_config();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()
                .expect("PORT must be a number"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://flick:flick_dev@localhost:5432/flick".into()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            cors_origin: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:4321".into()),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "flick-dev-secret-change-me".into()),
            auth_mode,
            saml,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
        }
    }

    fn read_saml_config() -> Option<SamlConfig> {
        let entry_point = env::var("SAML_ENTRY_POINT").ok()?;
        let issuer = env::var("SAML_ISSUER").ok()?;
        let idp_cert = env::var("SAML_IDP_CERT").ok()?;
        let callback_url = env::var("SAML_CALLBACK_URL").ok()?;

        Some(SamlConfig {
            entry_point,
            issuer,
            idp_cert,
            callback_url,
            attr_email: env::var("SAML_ATTR_EMAIL").unwrap_or_else(|_| "email".into()),
            attr_name: env::var("SAML_ATTR_NAME").unwrap_or_else(|_| "name".into()),
        })
    }
}
