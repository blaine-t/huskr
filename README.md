# Huskr
We made Huskr! In order to connect students on campus, we've made an online app to match each other! 
You can easily create a profile through your UNL SSO and account! This way all students can chat and message each other safely.

## Features

- **Microsoft SSO** — sign in with your university account, no password to manage
- **Smart feed** — profiles scored by shared interests, major, and RSO status
- **Swipe to match** — mutual likes create a match instantly
- **Real-time chat** — message your matches with text and image support
- **Profile setup** — photo upload, bio, major, age, and custom interest tags

## Stack

| Layer | Tech |
|---|---|
| Backend | Rust · Axum 0.8 |
| Auth | Microsoft Entra ID (OAuth2 + PKCE) · `axum-login` |
| Database | SQLite · `sqlx` (compile-time checked queries) |
| File storage | Local filesystem via `object_store` |
| Frontend | Vanilla JS · ES modules · no bundler |

## Running Locally

### Prerequisites

- Rust (stable)
- A Microsoft Entra ID app registration with a redirect URI of `http://localhost:48757/auth/callback`

### Setup

```bash
cp backend/.env.example backend/.env
# Fill in AZURE_CLIENT_ID, AZURE_CLIENT_SECRET, AZURE_TENANT_ID
```

### Start the server

```bash
cd backend
cargo run --bin backend
```

The server listens on `http://localhost:48757` and serves the frontend automatically.

### Seed mock data

```bash
cd backend
cargo run --bin seed
```

Inserts 8 mock users with interests, likes, matches, and messages so you can explore the UI without going through OAuth.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:app.db` | SQLite database path |
| `AZURE_CLIENT_ID` | — | Azure App Registration client ID |
| `AZURE_CLIENT_SECRET` | — | Azure client secret |
| `AZURE_TENANT_ID` | `common` | Tenant ID (`common` allows any university) |
| `REDIRECT_URL` | — | OAuth callback URL (must match Azure registration) |
| `FRONTEND_URL` | `http://localhost:48757` | Used for CORS and post-auth redirects |
| `OBJECT_STORE_PATH` | `./uploads` | Directory for uploaded images |

## Project Structure

```
backend/
  src/
    main.rs          # Router, middleware, server boot
    api/
      profiles.rs    # Compatible feed + profile lookup
      likes.rs       # Like/pass + match creation
      messages.rs    # Chat send/receive + image delivery
      matches.rs     # Match list
      user.rs        # Profile update (multipart)
    auth/            # Microsoft OAuth flow
    models.rs        # Domain types
    db.rs            # Pool init + migrations
  migrations/        # SQLite schema
  bin/seed.rs        # Mock data seeder

frontend/
  js/
    views/           # login, feed, profile, messages
    components/      # navbar, card, toast, match modal
    api.js           # fetch wrappers
    router.js        # Hash-based client-side router
  css/               # Scoped stylesheets per view
```
