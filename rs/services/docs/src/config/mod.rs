use std::env;

// Re-export from service-discovery crate
pub use service_discovery::ServiceConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_config: ApiConfig,
    pub port: u16,
    pub services: Vec<ServiceConfig>,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub title: String,
    pub description: String,
    pub version: String,
    pub url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let api_config = ApiConfig {
            title: env::var("API_TITLE").unwrap_or("API Documentation".to_string()),
            description: env::var("API_DESCRIPTION").unwrap_or("".to_string()),
            version: env::var("API_VERSION").unwrap_or("1.0.0".to_string()),
            url: env::var("API_URL").map_err(|_| anyhow::anyhow!("API_URL is required"))?,
        };

        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8000);

        let services = Self::discover_services()?;

        Ok(Config {
            api_config,
            port,
            services,
        })
    }

    fn discover_services() -> Result<Vec<ServiceConfig>, anyhow::Error> {
        // Get exclusion list from env (default: "docs,db,gateway")
        let exclude_str =
            env::var("EXCLUDE_SERVICES").unwrap_or_else(|_| "docs,db,gateway".to_string());
        let exclude_services: Vec<String> = exclude_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Get service name prefix from env (e.g., "int-" for internal services)
        let service_name_prefix = env::var("SERVICE_NAME_PREFIX").unwrap_or_default();

        // Determine domain suffix based on environment (Railway vs Docker Compose)
        let domain_suffix = if env::var("RAILWAY_ENVIRONMENT_NAME").is_ok() {
            ".railway.internal"
        } else {
            ""
        };

        // Use the service-discovery crate to discover services
        let options = service_discovery::DiscoveryOptions::new("./services")
            .exclude_services(exclude_services)
            .service_name_prefix(service_name_prefix)
            .domain_suffix(domain_suffix)
            .service_port(8000);

        Ok(service_discovery::discover_services(&options)?)
    }
}
