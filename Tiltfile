# Tiltfile

# Load environment variables from .env
load('ext://dotenv', 'dotenv')
dotenv()

# --- Service Discovery ---

def discover_services():
    """
    Auto-discover services from rs/services directory.
    Returns a dict mapping service names to their configuration.
    """
    services = {}

    # List all directories in rs/services
    service_dirs_output = local("ls -1 rs/services 2>/dev/null || true", quiet=True)
    service_dirs = [d.strip() for d in str(service_dirs_output).strip().split('\n') if d.strip()]

    for service_name in service_dirs:
        if not service_name:
            continue

        service_path = "rs/services/" + service_name

        # Check if Cargo.toml exists (indicates it's a buildable service)
        cargo_toml_check = local("test -f {}/Cargo.toml && echo 'true' || echo 'false'".format(service_path), quiet=True)
        if str(cargo_toml_check).strip() != 'true':
            continue

        # Default configuration
        config = {
            'name': service_name,
            'path': service_path,
            'bin': service_name,
            'features': [],
            'env_file': service_path + '/.env',
            'depends_on': ['postgres', 'redis'],
            'ports': [],
            'environment': {},
            'is_task': False,
            'trigger_mode': None,
            'variants': [],  # For internal API variants
        }

        # Check for optional service.json configuration
        service_json_path = "{}/service.json".format(service_path)
        service_json_check = local("test -f {} && echo 'true' || echo 'false'".format(service_json_path), quiet=True)
        if str(service_json_check).strip() == 'true':
            service_json_content = local("cat {}".format(service_json_path), quiet=True)
            custom_config = decode_json(str(service_json_content))
            # Merge custom config into default config
            for key, value in custom_config.items():
                config[key] = value

        services[service_name] = config

    return services

def discover_ts_services():
    """
    Auto-discover TypeScript services from ts/services directory.
    Returns a dict mapping service names to their configuration.
    """
    services = {}

    # List all directories in ts/services
    service_dirs_output = local("ls -1 ts/services 2>/dev/null || true", quiet=True)
    service_dirs = [d.strip() for d in str(service_dirs_output).strip().split('\n') if d.strip()]

    for service_name in service_dirs:
        if not service_name:
            continue

        service_path = "ts/services/" + service_name

        # Check if package.json exists (indicates it's a buildable service)
        pkg_json_check = local("test -f {}/package.json && echo 'true' || echo 'false'".format(service_path), quiet=True)
        if str(pkg_json_check).strip() != 'true':
            continue

        # Default configuration
        config = {
            'name': service_name,
            'path': service_path,
            'type': 'ts',
            'env_file': service_path + '/.env.local',
            'depends_on': ['gateway'],
            'ports': [],
            'environment': {},
            'is_task': False,
            'trigger_mode': None,
        }

        # Check for optional service.json configuration
        service_json_path = "{}/service.json".format(service_path)
        service_json_check = local("test -f {} && echo 'true' || echo 'false'".format(service_json_path), quiet=True)
        if str(service_json_check).strip() == 'true':
            service_json_content = local("cat {}".format(service_json_path), quiet=True)
            custom_config = decode_json(str(service_json_content))
            for key, value in custom_config.items():
                config[key] = value

        services[service_name] = config

    return services

def generate_ts_service_yaml(service_name, config):
    """
    Generate YAML lines for a single TypeScript service.
    """
    lines = []
    lines.append("  {}:".format(service_name))
    lines.append("    build:")
    lines.append("      context: ../ts")
    lines.append("      dockerfile: services/{}/Dockerfile.dev".format(config['name']))

    lines.append("    volumes:")
    lines.append("      - ../ts:/workspace")
    lines.append("      - {}_node_modules:/workspace/node_modules".format(service_name))
    lines.append("      - {}_svc_node_modules:/workspace/services/{}/node_modules".format(service_name, config['name']))
    lines.append("      - {}_next_cache:/workspace/services/{}/.next".format(service_name, config['name']))

    # depends_on
    if config.get('depends_on'):
        lines.append("    depends_on:")
        for dep in config['depends_on']:
            lines.append("      {}:".format(dep))
            lines.append("        condition: service_healthy")

    # env_file
    env_file_exists = local("test -f {} && echo 'true' || echo 'false'".format(config['env_file']), quiet=True)
    if str(env_file_exists).strip() == 'true':
        lines.append("    env_file:")
        lines.append("      - ../{}".format(config['env_file']))

    # environment variables
    env_vars = dict(config.get('environment', {}))
    if env_vars:
        lines.append("    environment:")
        for key, value in env_vars.items():
            lines.append("      {}: {}".format(key, value))

    # ports
    if config.get('ports'):
        lines.append("    ports:")
        for port in config['ports']:
            lines.append("      - \"{}\"".format(port))

    # command
    lines.append("    working_dir: /workspace")
    lines.append("    command:")
    lines.append("      - pnpm")
    lines.append("      - --filter")
    lines.append("      - {}".format(config['name']))
    lines.append("      - dev")

    # healthcheck for web services
    if not config.get('is_task') and config.get('ports'):
        # Extract container port from first port mapping
        first_port = str(config['ports'][0]).split(':')[-1]
        lines.append("    healthcheck:")
        lines.append("      test: [\"CMD-SHELL\", \"timeout 1 bash -c '</dev/tcp/localhost/{}' || exit 1\"]".format(first_port))
        lines.append("      interval: 10s")
        lines.append("      timeout: 5s")
        lines.append("      retries: 5")
        lines.append("      start_period: 30s")

    return lines

def generate_docker_compose_yaml(services_config, ts_services_config=None):
    """
    Generate docker-compose YAML content as a string.
    """
    yaml_lines = [
        "include:",
        "  - ../docker-compose.yml",
        "",
        "services:"
    ]

    # Rust services
    for service_name, config in services_config.items():
        yaml_lines.extend(generate_service_yaml(service_name, config))

        for variant in config.get('variants', []):
            variant_name = variant['name']
            variant_config = dict(config)
            for key, value in variant.items():
                variant_config[key] = value
            yaml_lines.extend(generate_service_yaml(variant_name, variant_config))

    # TypeScript services
    if ts_services_config:
        for service_name, config in ts_services_config.items():
            yaml_lines.extend(generate_ts_service_yaml(service_name, config))

    # Add volumes section
    yaml_lines.append("")
    yaml_lines.append("volumes:")

    for service_name in services_config.keys():
        yaml_lines.append("  {}_target:".format(service_name))
        yaml_lines.append("  {}_cargo_cache:".format(service_name))

        for variant in services_config[service_name].get('variants', []):
            variant_name = variant['name']
            yaml_lines.append("  {}_target:".format(variant_name))
            yaml_lines.append("  {}_cargo_cache:".format(variant_name))

    if ts_services_config:
        for service_name in ts_services_config.keys():
            yaml_lines.append("  {}_node_modules:".format(service_name))
            yaml_lines.append("  {}_svc_node_modules:".format(service_name))
            yaml_lines.append("  {}_next_cache:".format(service_name))

    return "\n".join(yaml_lines)

def generate_service_yaml(service_name, config):
    """
    Generate YAML lines for a single service.
    """
    lines = []
    lines.append("  {}:".format(service_name))
    lines.append("    build:")
    lines.append("      context: ../rs")
    lines.append("      target: dev")

    if config.get('build_arg', True):
        lines.append("      args:")
        lines.append("        SERVICE: {}".format(config['bin']))

    lines.append("    volumes:")
    lines.append("      - ../rs:/workspace")
    lines.append("      - {}_target:/workspace/target".format(service_name))
    lines.append("      - {}_cargo_cache:/usr/local/cargo/registry".format(service_name))

    # depends_on
    lines.append("    depends_on:")
    for dep in config.get('depends_on', []):
        lines.append("      {}:".format(dep))
        lines.append("        condition: service_healthy")

    # env_file
    env_file_exists = local("test -f {} && echo 'true' || echo 'false'".format(config['env_file']), quiet=True)
    if str(env_file_exists).strip() == 'true':
        lines.append("    env_file:")
        lines.append("      - ../{}".format(config['env_file']))

    # environment variables
    if config.get('environment'):
        lines.append("    environment:")
        for key, value in config['environment'].items():
            lines.append("      {}: {}".format(key, value))

    # ports
    if config.get('ports'):
        lines.append("    ports:")
        for port in config['ports']:
            lines.append("      - \"{}\"".format(port))

    # command
    cmd = "run --bin {}".format(config['bin'])
    if config.get('features'):
        cmd += " --features " + ",".join(config['features'])

    lines.append("    command:")
    lines.append("      - cargo")
    lines.append("      - watch")
    lines.append("      - -x")
    lines.append("      - \"{}\"".format(cmd))

    # restart policy
    if config.get('is_task'):
        lines.append("    restart: \"no\"")

    # healthcheck
    if not config.get('is_task'):
        lines.append("    healthcheck:")
        lines.append("      test: [\"CMD-SHELL\", \"timeout 1 bash -c '</dev/tcp/localhost/8000' || exit 1\"]")
        lines.append("      interval: 10s")
        lines.append("      timeout: 5s")
        lines.append("      retries: 5")
        lines.append("      start_period: 60s")

    return lines

# Discover services
print("Discovering services in rs/services...")
services = discover_services()
print("Found {} Rust services: {}".format(len(services), ", ".join(sorted(services.keys()))))

print("Discovering services in ts/services...")
ts_services = discover_ts_services()
if ts_services:
    print("Found {} TypeScript services: {}".format(len(ts_services), ", ".join(sorted(ts_services.keys()))))
else:
    print("No TypeScript services found (ts/services/ may not exist yet)")

# Generate docker-compose file
yaml_content = generate_docker_compose_yaml(services, ts_services if ts_services else None)
local("mkdir -p .tilt")
local("cat > .tilt/docker-compose.yml << 'YAML_EOF'\n{}\nYAML_EOF".format(yaml_content))

# Load the generated docker-compose file (which includes docker-compose.yml)
docker_compose(".tilt/docker-compose.yml")

# --- Infrastructure Resources ---
dc_resource(
    "postgres",
    labels=["infra"],
)

dc_resource(
    "redis",
    labels=["infra"],
)

dc_resource(
    "minio",
    labels=["infra"],
)

# --- Create Tilt Resources ---

for service_name, config in services.items():
    # Resource dependencies
    resource_deps = config.get('depends_on', ['postgres', 'redis'])

    # If service depends on minio, also depend on minio-init
    if 'minio' in resource_deps:
        resource_deps = resource_deps + ['minio-init']

    # Links for API services
    links = []
    if not config.get('is_task'):
        if service_name == 'gateway':
            links = [
                link("http://localhost:8000/", "docs"),
                link("http://localhost:8000/openapi.json", "openapi spec"),
                link("http://localhost:8000/health", "health"),
            ]
        else:
            links = [
                link("http://localhost:8000/health", "health (via gateway)"),
            ]

    # Create resource
    dc_resource_args = {
        'labels': ['tasks'] if config.get('is_task') else ['services'],
        'resource_deps': resource_deps,
    }

    if links:
        dc_resource_args['links'] = links

    if config.get('trigger_mode'):
        dc_resource_args['trigger_mode'] = TRIGGER_MODE_MANUAL

    dc_resource(service_name, **dc_resource_args)

    # Create variants
    for variant in config.get('variants', []):
        variant_name = variant['name']
        variant_deps = variant.get('depends_on', resource_deps)
        variant_links = variant.get('links', [])

        if variant_name == 'int-gateway':
            variant_links = [
                link("http://localhost:8001/", "docs"),
                link("http://localhost:8001/openapi.json", "openapi spec"),
                link("http://localhost:8001/health", "health"),
            ]

        variant_args = {
            'labels': ['services'],
            'resource_deps': variant_deps,
        }

        if variant_links:
            variant_args['links'] = variant_links

        dc_resource(variant_name, **variant_args)

# --- TypeScript Service Resources ---

for service_name, config in ts_services.items():
    resource_deps = config.get('depends_on', ['gateway'])

    links = []
    if not config.get('is_task') and config.get('ports'):
        first_port = str(config['ports'][0]).split(':')[0]
        links = [
            link("http://localhost:{}/".format(first_port), "app"),
        ]

    dc_resource_args = {
        'labels': ['frontend'],
        'resource_deps': resource_deps,
    }

    if links:
        dc_resource_args['links'] = links

    if config.get('trigger_mode'):
        dc_resource_args['trigger_mode'] = TRIGGER_MODE_MANUAL

    dc_resource(service_name, **dc_resource_args)

# --- Initialization Tasks ---

local_resource(
    "minio-init",
    cmd='docker exec tilt-minio-1 sh -c "mc alias set local http://localhost:9000 {} {} && mc mb local/evmauth-assets --ignore-existing"'.format(
        os.getenv("MINIO_ROOT_USER"),
        os.getenv("MINIO_ROOT_PASSWORD"),
    ),
    labels=["infra"],
    resource_deps=["minio"],
)

# --- Manual Tasks ---

local_resource(
    "sqlx-prepare",
    cmd='DATABASE_URL="postgres://{}:{}@localhost:{}/{}" cargo sqlx prepare --workspace'.format(
        os.getenv("POSTGRES_USER"),
        os.getenv("POSTGRES_PASSWORD"),
        os.getenv("POSTGRES_PORT"),
        os.getenv("POSTGRES_DB"),
    ),
    dir="rs",
    labels=["tasks"],
    resource_deps=["postgres"],
    trigger_mode=TRIGGER_MODE_MANUAL,
    auto_init=False,
)
