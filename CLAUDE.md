# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Huskr** is a university student friend making app. The repository currently contains only a Rust backend (`backend/`). A Dioxus SPA frontend was previously present but has been removed.

## Backend Commands

All commands run from the `backend/` directory.

```bash
# Run the server
cargo run --bin backend

# Build
cargo build

# Check without building
cargo check

# Lint
cargo clippy

# Run migrations + seed with mock data
cargo run --bin seed
```

The server listens on `0.0.0.0:48757`.

## Environment Setup

Copy/edit `backend/.env`. Required variables:
- `DATABASE_URL` — defaults to `sqlite:app.db`
- `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID` — Microsoft Entra ID OAuth app credentials
- `REDIRECT_URL` — OAuth callback URL, must match Azure App Registration (e.g. `http://localhost:48757/auth/callback`)
- `FRONTEND_URL` — Base URL of the SPA frontend, used for CORS and post-auth redirects (default: `http://localhost:8080`)
- `OBJECT_STORE_PATH` — local path for uploaded images (default: `./uploads`)

Migrations run automatically on startup via `sqlx::migrate!("./migrations")`.

## Architecture

### Tech Stack
- **Web framework**: Axum 0.8
- **Auth**: `axum-login` with Microsoft Entra ID (OAuth2 + PKCE + CSRF)
- **Database**: SQLite via `sqlx` with compile-time checked queries
- **File storage**: `object_store` (local filesystem; key `profiles/{user_id}`)
- **Sessions**: `tower-sessions` with in-memory store (swap for persistent store in production)

### Module Structure (`backend/src/`)

| Module | Purpose |
|---|---|
| `main.rs` | Router setup, middleware wiring, CORS, server boot |
| `lib.rs` | `AppState` definition (`SqlitePool`, `MicrosoftBackend`, `frontend_url`, `ObjectStore`) |
| `models.rs` | All domain structs: `User`, `UserResponse`, `Match`, `Message`, `Like`, interests |
| `db.rs` | Pool initialization + runs migrations |
| `error.rs` | `AppError` enum with `IntoResponse` impl |
| `middleware.rs` | `require_user` — 401 guard applied to all protected routes |
| `auth/backend.rs` | `MicrosoftBackend`: implements `AuthnBackend`, does token exchange, upserts user in DB |
| `auth/routes.rs` | `/auth/login`, `/auth/callback`, `/auth/logout` |
| `api/user.rs` | `GET /user/me`, `POST /user/profile` (multipart: bio, image, major, age, interests) |
| `api/profiles.rs` | `GET /profiles/compatible`, `GET /profiles/{id}`, `GET /profiles/{id}/image` |
| `api/likes.rs` | `POST /like` — upserts like/pass, creates match on mutual like |
| `api/messages.rs` | `GET /messages/{user_id}`, `POST /message` |
| `bin/seed.rs` | Idempotent seeder: 8 mock users with interests, likes, matches, messages |

### API Routes

Public:
- `GET /auth/login?next=/path` — redirects to Microsoft SSO
- `GET /auth/callback` — OAuth callback, establishes session
- `GET /auth/logout`

Protected (require authenticated session):
- `GET /user/me`
- `POST /user/profile` (multipart/form-data)
- `POST /like` — `{ liked_id, is_like }`
- `POST /message` — `{ recipient_id, content }`
- `GET /messages/{user_id}`
- `GET /profiles/compatible` — scored by shared interests (+3), same major (+2), same RSO status (+1), excludes already-acted-on users
- `GET /profiles/{id}`
- `GET /profiles/{id}/image`

### Database Schema

Core tables: `users`, `interests`, `user_interests` (junction), `matches` (user1_id < user2_id enforced by CHECK), `messages`, `likes`.

Migrations are in `backend/migrations/` numbered `0001`–`0004`.

### Testing

`backend/index.html` is a standalone browser-based API test harness — open it directly in a browser while the backend is running.
