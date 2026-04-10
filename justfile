# =========================
# 🔧 BASIC
# =========================

run:
    cargo run

build:
    cargo build

release:
    cargo build --release

clean:
    cargo clean

# =========================
# 🧪 DEV
# =========================

dev:
    cargo watch -x run

fmt:
    cargo fmt

lint:
    cargo clippy -- -D warnings

# =========================
# 🐳 DOCKER
# =========================

docker-build:
    docker-compose build

docker-up:
    docker-compose up

docker-up-detached:
    docker-compose up -d

docker-down:
    docker-compose down

docker-logs:
    docker-compose logs -f

docker-restart:
    docker-compose down && docker-compose up --build

# =========================
# 🗄️ DATABASE
# =========================

db-up:
    docker-compose up postgres_db

db-reset:
    docker-compose down -v && docker-compose up -d postgres_db

db-shell:
    docker exec -it postgres_db psql -U wayve_user -d wayve_db

# =========================
# 🔐 SQLx
# =========================

sqlx-prepare:
    DATABASE_URL=postgres://wayve_user:wayve_password@localhost:5432/wayve_db cargo sqlx prepare

sqlx-migrate:
    sqlx migrate run

# =========================
# 📧 BACKEND + FRONTEND
# =========================

start-all:
    docker-compose up --build

restart-backend:
    docker-compose restart backend

restart-nginx:
    docker-compose restart nginx

# =========================
# 🔥 GMAIL SYNC DEBUG
# =========================

logs-backend:
    docker logs -f backend

# =========================
# ⚡ SHORTCUTS
# =========================

r: run
d: dev
u: docker-up
ud: docker-up-detached
dd: docker-down