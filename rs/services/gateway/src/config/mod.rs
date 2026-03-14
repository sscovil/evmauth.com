use std::env;

// Re-export from service-discovery crate
pub use service_discovery::ServiceConfig;

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_GATEWAY_TIMEOUT_SECS: u64 = 30;
const DEFAULT_EXCLUDE_SERVICES: &str = "gateway,db";
const DEFAULT_SERVICE_PORT: u16 = 8000;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub timeout_secs: u64,
    pub services: Vec<ServiceConfig>,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_PORT);

        let timeout_secs = env::var("GATEWAY_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_GATEWAY_TIMEOUT_SECS);

        let services = Self::discover_services()?;

        Ok(Config {
            port,
            timeout_secs,
            services,
        })
    }

    fn discover_services() -> Result<Vec<ServiceConfig>, anyhow::Error> {
        // Get exclusion list from env (default: "gateway,db")
        let exclude_str =
            env::var("EXCLUDE_SERVICES").unwrap_or_else(|_| DEFAULT_EXCLUDE_SERVICES.to_string());
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
            .service_port(DEFAULT_SERVICE_PORT);

        Ok(service_discovery::discover_services(&options)?)
    }
}
