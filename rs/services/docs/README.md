# OpenAPI Docs

Unified API documentation service that automatically discovers and aggregates OpenAPI specifications from all services.

## Features

- **Auto-Discovery**: Automatically scans `rs/services/` directory to find services
- **Spec Aggregation**: Fetches and merges OpenAPI specs from all discovered services
- **Environment-Aware**: Adapts service URLs for Docker Compose and Railway deployments
- **Swagger UI**: Interactive API documentation interface
- **Graceful Degradation**: Continues to serve available specs even if some services are unavailable

## API Endpoints

- `GET /` - Swagger UI interface
- `GET /openapi.json` - Merged OpenAPI specification
- `GET /services` - List of discovered services with URLs
- `GET /health` - Health check

## Service Discovery

The service automatically discovers APIs by scanning the `rs/services/` directory. Services are included if they:

1. Have a directory in `rs/services/`
2. Are not in the exclusion list (defaults: `docs`, `db`)
3. Expose an OpenAPI spec at `/openapi.json`

### URL Resolution

Service URLs are automatically resolved based on the environment:

- **Docker Compose**: `http://<service>:8000/openapi.json`
- **Railway**: `http://<service>.railway.internal:8000/openapi.json`

Detection is based on the presence of the `RAILWAY_ENVIRONMENT_NAME` environment variable.

## Configuration

Environment variables:

- `API_GATEWAY_URL` - Base URL of the API gateway (required, e.g., "https://api.evmauth.com" or "http://localhost:8000")
- `EXCLUDE_SERVICES` - Comma-separated list of services to exclude (default: "docs,db")
- `PORT` - Port to listen on (default: 8000)

## Spec Merging

When services are discovered, their OpenAPI specs are merged:

- Paths are prefixed with service names (e.g., `/people` becomes `/auth/people`)
- The `servers` field is set to the configured `API_GATEWAY_URL`
- Component schemas are prefixed with service names to avoid conflicts
- Tags are prefixed with service names for clarity

This ensures the documentation accurately reflects how the APIs will be accessed through the API gateway.

## Running

```bash
cargo run -p docs
```

Run checks (format, lint, test, build) from the repository root:

```bash
./check.sh rs/services/docs
```
