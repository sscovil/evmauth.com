# API Gateway

API gateway that serves as the single public entry point to all backend microservices. Routes requests to backend services based on path prefixes and provides service discovery, health checks, and request forwarding.

## Features

- **Automatic Service Discovery**: Scans `rs/services/` directory to discover available services
- **Path-Based Routing**: Routes requests like `/auth/people` to `http://auth:8000/people`
- **Environment-Aware**: Adapts service URLs for Docker Compose and Railway deployments
- **Health Aggregation**: Checks health of all backend services
- **Pass-Through Proxy**: Forwards HTTP method, headers, query parameters, and request body
- **Error Handling**: Maps backend errors to appropriate HTTP status codes

## Service Discovery

The gateway automatically discovers backend services by scanning the `rs/services/` directory. Services are included if they:

1. Have a directory in `rs/services/`
2. Are not in the exclusion list (defaults: `gateway`, `db`)
3. Are running and accessible on port 8000

### URL Resolution

Service URLs are automatically resolved based on the environment:

- **Docker Compose**: `http://<service>:8000`
- **Railway**: `http://<service>.railway.internal:8000`

Detection is based on the presence of the `RAILWAY_ENVIRONMENT_NAME` environment variable.

## Path Routing

The gateway extracts the service name from the first path segment and forwards the request:

```
Incoming:  /auth/people
Forwarded: http://auth:8000/people

Incoming:  /openapi.json
Forwarded: http://docs:8000/openapi.json
```

## Configuration

Environment variables:

- `PORT` - Port to listen on (default: 8000)
- `EXCLUDE_SERVICES` - Comma-separated list of services to exclude (default: "gateway,db")
- `GATEWAY_TIMEOUT_SECS` - Request timeout in seconds (default: 30)
- `RAILWAY_ENVIRONMENT_NAME` - Set by Railway platform (auto-detected)

## Error Handling

- `404 Not Found` - Unknown service in path
- `400 Bad Request` - Invalid request path
- `502 Bad Gateway` - Backend service unavailable or error
- `504 Gateway Timeout` - Request timeout exceeded

## Running

```bash
cargo run -p gateway
```

Run checks (format, lint, test, build) from the repository root:

```bash
./check.sh rs/services/gateway
```
