# Environments

The project now has explicit development and production configuration paths. Real env files are ignored by git; use the checked-in `.example` files as templates.

## Development

Create local env files:

```sh
cp .env.development.example .env.development
cp backend/.env.development.example backend/.env.development
cp frontend/.env.development.example frontend/.env.development
cp infra/.env.development.example infra/.env.development
```

Start the development stack from the repo root:

```sh
docker compose -f infra/docker-compose.dev.yml --env-file infra/.env.development up --build
```

Or, if your current directory is `infra/`, use paths relative to `infra/`:

```sh
docker compose -f docker-compose.dev.yml --env-file .env.development up --build
```

Development runs:

- frontend: Vite dev server in `frontend/Dockerfile.dev`
- nginx: `infra/nginx/nginx.dev.conf.template`
- database env: `.env.development`
- backend env: `backend/.env.development`
- frontend env: `frontend/.env.development`
- infra/nginx env: `infra/.env.development`
- optional workers: add `--profile workers`

The dev CSP allows Vite's inline style/script behavior. Use the production stack to verify strict CSP behavior.

## macOS Local Development

For the fastest local loop on macOS, run Postgres/Redis in Docker and run the frontend/backend directly on the host:

```sh
cd infra
docker compose -f docker-compose.dev.yml --env-file .env.development up -d postgres_db redis mailhog
```

Run the backend:

```sh
cd backend
RWAYVE_ENV=development cargo run
```

Run the frontend:

```sh
cd frontend
npm run dev
```

For Google OAuth in this local setup, your Google Cloud OAuth client must include this exact authorized redirect URI:

```text
http://localhost:8080/oauth/callback
```

The backend receives the callback on port `8080`, then sends the browser back to the Vite app at `http://localhost:5173`.

## Production

Create production env files:

```sh
cp .env.production.example .env.production
cp backend/.env.production.example backend/.env.production
cp frontend/.env.production.example frontend/.env.production
cp infra/.env.production.example infra/.env.production
```

Before starting production, replace every placeholder secret and host value. Generate strong values, for example:

```sh
openssl rand -hex 64
openssl rand -hex 32
```

Start the production stack from the repo root:

```sh
docker compose -f infra/docker-compose.prod.yml --env-file infra/.env.production up -d --build
```

Or, if your current directory is `infra/`, use:

```sh
docker compose -f docker-compose.prod.yml --env-file .env.production up -d --build
```

Production runs:

- frontend: static Vite build in `frontend/Dockerfile.prod`, served by `frontend/nginx.prod.conf`
- nginx: `infra/nginx/nginx.prod.conf.template`
- database env: `.env.production`
- backend env: `backend/.env.production`
- frontend env/build args: `frontend/.env.production` and `infra/.env.production`
- infra/nginx env: `infra/.env.production`
- backend workers: enabled by default

Production does not publish Postgres, Redis, backend, or frontend directly to the host. Only nginx publishes port `80`.

## Local Backend Without Docker

The backend loads env files in this order when running locally:

1. `.env`
2. `ENV_FILE`, when set
3. `.env.${RWAYVE_ENV}` or `.env.${ENV}`
4. `backend/.env.${RWAYVE_ENV}` or `backend/.env.${ENV}`
5. `backend/.env`

For an explicit local production-style run:

```sh
ENV_FILE=backend/.env.production cargo run
```
