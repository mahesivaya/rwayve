# syntax=docker/dockerfile:1.7
FROM node:20-bookworm-slim

WORKDIR /app

# Use BuildKit's npm cache so re-installs reuse the cache between builds.
COPY package*.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm install

COPY . .

EXPOSE 5173
CMD ["npm", "run", "dev", "--", "--host"]
