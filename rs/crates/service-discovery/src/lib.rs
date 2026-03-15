use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Errors that can occur during service discovery
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("failed to read services manifest: {0}")]
    ManifestRead(#[from] std::io::Error),
}

/// Configuration for a discovered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// The service name as it appears in the manifest (e.g., "auth", "wallets").
    pub name: String,
    /// The fully qualified base URL used to reach the service (e.g., "http://auth:8000").
    pub base_url: String,
}

impl ServiceConfig {
    /// Create a new ServiceConfig with the given name and base URL
    pub fn new(name: String, base_url: String) -> Self {
        Self { name, base_url }
    }

    /// Get the health check URL for this service
    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url)
    }

    /// Get the OpenAPI spec URL for this service
    pub fn openapi_url(&self) -> String {
        format!("{}/openapi.json", self.base_url)
    }
}

/// Options for service discovery
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    /// Path to the services directory (relative to workspace root)
    pub services_path: String,
    /// Services to exclude from discovery
    pub exclude_services: Vec<String>,
    /// Prefix to prepend to service names (e.g., "int-" for internal services)
    pub service_name_prefix: String,
    /// Domain suffix to append to service names (e.g., ".railway.internal" or "")
    pub domain_suffix: String,
    /// Port that services listen on
    pub service_port: u16,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            services_path: "./services".to_string(),
            exclude_services: vec![],
            service_name_prefix: String::new(),
            domain_suffix: String::new(),
            service_port: 8000,
        }
    }
}

impl DiscoveryOptions {
    /// Create a new DiscoveryOptions with the given services path
    pub fn new(services_path: impl Into<String>) -> Self {
        Self {
            services_path: services_path.into(),
            ..Default::default()
        }
    }

    /// Set the services to exclude from discovery
    pub fn exclude_services(mut self, services: Vec<String>) -> Self {
        self.exclude_services = services;
        self
    }

    /// Set the service name prefix (e.g., "int-" for internal service variants)
    pub fn service_name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.service_name_prefix = prefix.into();
        self
    }

    /// Set the domain suffix (e.g., ".railway.internal" for Railway deployments)
    pub fn domain_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.domain_suffix = suffix.into();
        self
    }

    /// Set the port that services listen on
    pub fn service_port(mut self, port: u16) -> Self {
        self.service_port = port;
        self
    }
}

/// Discover services from manifest file
///
/// Returns a list of ServiceConfig with base URLs (without trailing paths)
pub fn discover_services(options: &DiscoveryOptions) -> Result<Vec<ServiceConfig>, DiscoveryError> {
    let manifest_path = Path::new("/app/services-manifest.txt");
    let mut services = Vec::new();

    // Read service names from manifest file
    let contents = fs::read_to_string(manifest_path)?;

    for line in contents.lines() {
        let name = line.trim();

        // Skip empty lines, hidden directories, and excluded services
        if name.is_empty()
            || name.starts_with('.')
            || options.exclude_services.contains(&name.to_string())
        {
            continue;
        }

        services.push(ServiceConfig::new(
            name.to_string(),
            format!(
                "http://{}{}{}:{}",
                options.service_name_prefix, name, options.domain_suffix, options.service_port
            ),
        ));
    }

    Ok(services)
}

/// Health check result for a single service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// The name of the service that was checked.
    pub name: String,
    /// Whether the service responded successfully to its health endpoint.
    pub status: HealthStatus,
}

/// Health status of a discovered service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is responding normally
    Healthy,
    /// Service is not responding or returning errors
    Unhealthy,
}

impl HealthStatus {
    /// Returns true if the service is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Aggregated health status for all services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHealth {
    /// The overall status across all services (healthy only if every service is healthy).
    pub status: OverallStatus,
    /// Map of service name to its health status string (e.g., "healthy" or "unhealthy").
    pub services: HashMap<String, String>,
}

/// Aggregated health status across all services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverallStatus {
    /// All services are healthy
    Healthy,
    /// One or more services are unhealthy
    Degraded,
}

impl OverallStatus {
    /// Returns true if all services are healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, OverallStatus::Healthy)
    }
}

/// Check the health of a single service
///
/// Tries multiple health endpoint paths (e.g., /health, /api/health) with a timeout
pub async fn check_service_health(
    client: &reqwest::Client,
    service: &ServiceConfig,
    health_paths: &[&str],
    timeout_secs: u64,
) -> ServiceHealth {
    let mut status = HealthStatus::Unhealthy;

    for path in health_paths {
        let url = format!("{}{}", service.base_url, path);

        match client
            .get(&url)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                status = HealthStatus::Healthy;
                break;
            }
            _ => continue,
        }
    }

    ServiceHealth {
        name: service.name.clone(),
        status,
    }
}

/// Check the health of all services
///
/// Returns an aggregated health status with individual service statuses
pub async fn check_all_services_health(
    client: &reqwest::Client,
    services: &[ServiceConfig],
    health_paths: &[&str],
    timeout_secs: u64,
) -> AggregatedHealth {
    let mut services_health = HashMap::new();

    for service in services {
        let health = check_service_health(client, service, health_paths, timeout_secs).await;
        services_health.insert(service.name.clone(), health.status.to_string());
    }

    let all_healthy = services_health.values().all(|s| s == "healthy");
    let status = if all_healthy {
        OverallStatus::Healthy
    } else {
        OverallStatus::Degraded
    };

    AggregatedHealth {
        status,
        services: services_health,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_urls() {
        let service = ServiceConfig::new("auth".to_string(), "http://auth:8000".to_string());

        assert_eq!(service.name, "auth");
        assert_eq!(service.base_url, "http://auth:8000");
        assert_eq!(service.health_url(), "http://auth:8000/health");
        assert_eq!(service.openapi_url(), "http://auth:8000/openapi.json");
    }

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Unhealthy.is_healthy());
    }

    #[test]
    fn test_discovery_options_builder() {
        let options = DiscoveryOptions::new("./services")
            .exclude_services(vec!["db".to_string(), "gateway".to_string()])
            .domain_suffix(".railway.internal")
            .service_port(8000);

        assert_eq!(options.services_path, "./services");
        assert_eq!(options.exclude_services, vec!["db", "gateway"]);
        assert_eq!(options.domain_suffix, ".railway.internal");
        assert_eq!(options.service_port, 8000);
    }

    #[test]
    fn test_overall_status() {
        assert!(OverallStatus::Healthy.is_healthy());
        assert!(!OverallStatus::Degraded.is_healthy());
    }
}
