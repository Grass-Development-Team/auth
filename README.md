# Madoka Auth

A simple authentication service for user registration, login, and role-based access control.

> [!WARNING]
> The program is still under development and may contain bugs or security vulnerabilities. **DO NOT USE IT IN PRODUCTION**.
> Use it at your own risk.

## TODO

- [] Basic Authentication
- [] Role-Based Access Control
- [] OIDC Authentication
- [] Multi-Factor Authentication
- [] Passcode & Other Authentication Methods
- [] User Management

## Getting Started

You'll need latest Rust, PostgreSQL and Redis to run this locally.

> [!NOTE]
> Older version maybe usable, but we DO NOT provide any support for it.

```bash
git clone <repository-url>
cd auth
just setup
just dev
```

The service starts on `http://localhost:7817`.

## Configuration

On first run, it creates a `config.toml` file. Update the database and Redis settings as needed.

When you first start the service, it creates a super admin account. Check the startup logs for the username and password.

## Development

Use these commands for development:

```bash
just dev          # Run in development mode
just dev-watch    # Auto-reload on file changes
just test         # Run tests
just fmt          # Format code
just clippy       # Check code quality
```

## Docker

The easiest way to run this is with Docker:

```bash
# Build the image
docker build -t auth-service .

# Run with default settings
docker run -p 7817:7817 auth-service

# Or use the justfile commands
just docker-build
just docker-run
```

For production, you'll want to mount your config file and connect to external databases:

```bash
docker run -d \
  -p 7817:7817 \
  -v /path/to/your/config.toml:/app/config.toml \
  -e LOG_LEVEL=INFO \
  auth-service
```

You can also use docker-compose. Here's a basic setup:

```yaml
version: '3.8'
services:
  auth:
    build: .
    ports:
      - "7817:7817"
    volumes:
      - ./config.toml:/app/config.toml
    depends_on:
      - postgres
      - redis

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: auth_db
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password

  redis:
    image: redis:7
```

## Contributing

Fork the repo, make your changes, run `just quality` to make sure everything looks good, then submit a pull request.

## License

MIT License
