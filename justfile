# Run backend locally
backend:
    cd backend && cargo run

# Run frontend (Vite)
frontend:
    cd frontend && npm run dev

# Run everything with Docker
up:
    docker compose up --build

# Stop containers
down:
    docker compose down

# Restart clean
restart:
    docker compose down
    docker compose up --build

# View logs
logs:
    docker compose logs -f

# Backend logs only
logs-backend:
    docker compose logs -f backend

# Rebuild backend only
rebuild-backend:
    docker compose build backend
    docker compose up -d backend

# Run migrations / DB check
db:
    docker exec -it postgres_db psql -U wayve_user -d wayve_db

# Clean Rust build
clean:
    cd backend && cargo clean

# Fix Rust warnings
fix:
    cd backend && cargo fix --allow-dirty

# Format code
fmt:
    cd backend && cargo fmt

# Lint
clippy:
    cd backend && cargo clippy

# Install frontend deps
install-frontend:
    cd frontend && npm install