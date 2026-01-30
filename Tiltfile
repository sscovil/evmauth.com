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

def generate_docker_compose_yaml(services_config):
    """
    Generate docker-compose YAML content as a string.
    """
    yaml_lines = [
        "include:",
        "  - ../docker-compose.yml",
        "",
        "services:"
    ]

    for service_name, config in services_config.items():
        # Generate main service
        yaml_lines.extend(generate_service_yaml(service_name, config))

        # Generate variants
        for variant in config.get('variants', []):
            variant_name = variant['name']
            variant_config = dict(config)
            for key, value in variant.items():
                variant_config[key] = value
            yaml_lines.extend(generate_service_yaml(variant_name, variant_config))

    # Add volumes section
    yaml_lines.append("")
    yaml_lines.append("volumes:")

    for service_name in services_config.keys():
        yaml_lines.append("  {}_target:".format(service_name))
        yaml_lines.append("  {}_cargo_cache:".format(service_name))

        # Add volumes for variants
        for variant in services_config[service_name].get('variants', []):
            variant_name = variant['name']
            yaml_lines.append("  {}_target:".format(variant_name))
            yaml_lines.append("  {}_cargo_cache:".format(variant_name))

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
print("Found {} services: {}".format(len(services), ", ".join(sorted(services.keys()))))

# Generate docker-compose file
yaml_content = generate_docker_compose_yaml(services)
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
