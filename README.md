# Relay

Relay is an open-source desktop app for AI chat, tasks, and coding. Use your own AI models, API keys, and server.

The app is in development. Chat comes first, with Work and Code planned for later.

Website: [relay.smbl.dev](https://relay.smbl.dev), deployed on Cloudflare.

## Tech stack

| Part | Stack | Status |
| --- | --- | --- |
| Landing page | Rust, Topcoat, WebAssembly, Cloudflare Workers, D1 | Deployed |
| Desktop app | Rust, GPUI, SQLite | Planned |
| Product API | Rust, Axum, SQLite | Planned |
| AI service | Python, FastAPI, PydanticAI | Planned |
| Task execution | Rust local executor; optional Modal cloud tasks | Planned |

## Current status

- The landing page is publicly deployed at [relay.smbl.dev](https://relay.smbl.dev), with a working D1 waitlist.
- Duplicate email signups are blocked, including simultaneous submissions. Verified on the public site.
- All seven integration tests pass on Workers and Pages. The deployment dry run passes.
- The desktop app, product API, and AI service have not been built yet.

See the [landing page README](apps/landing/README.md) for build, run, and self-hosting instructions. Deployment requires your own Cloudflare account and D1 IDs in environment variables; local settings are ignored by Git.
