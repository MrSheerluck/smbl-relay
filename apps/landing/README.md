# Relay landing page

The landing page is authored in Rust. Topcoat renders the HTML; the Cloudflare Rust SDK handles requests and D1; a separate Rust WebAssembly client handles the preview and waitlist form. The design, simple copy, R logo, and navigation remain the same.

Public site: [relay.smbl.dev](https://relay.smbl.dev), deployed on Cloudflare Workers with a production D1 waitlist.

## Source

- `src/lib.rs`: Topcoat page view, shared by server rendering and static export.
- `src/edge.rs`: Rust Worker routing, bounded request parsing, waitlist API, and D1 insert.
- `src/email.rs`: email validation and normalization.
- `client/src/lib.rs`: Rust browser events, keyboard navigation, sample prompts, and asynchronous waitlist submission.
- `src/bin/assemble.rs`: assembles assets and generates the browser loader and Pages adapter.
- `src/bin/cloudflare.rs`: reads deployment settings and runs Wrangler with your account and D1 binding.
- `migrations/0001_waitlist.sql`: case-insensitive unique email key and signup timestamp.
- `tests/waitlist.rs`: Rust integration tests against a real local D1 binding.

There are no handwritten JavaScript application or test files. `worker-build` and `wasm-bindgen` generate the JavaScript bridge that Cloudflare and browsers use to load WebAssembly. The assembly step also generates a three-line browser loader and a Pages adapter that forwards to the Rust handler. These generated files are ignored by version control. HTML, CSS, SVG, SQL, build shell scripts, and Wrangler configuration remain appropriate parts of the website.

Both Workers and Pages run the same Rust API and Topcoat renderer. Pages uses advanced mode with generated `_worker.js` modules. `dist/index.html` also contains a static export of the same view. No Leptos fallback is needed. This uses Topcoat's view renderer; it does not implement Topcoat's full router or reactive shard runtime.

The interactive Chat / Work / Code mockup does not send or save prompts and has no AI service attached. Relay remains in development.

## Waitlist

`POST /api/waitlist` trims and lowercases the email, validates it, and uses a parameterized D1 insert with `ON CONFLICT(email) DO NOTHING`. The database primary key prevents duplicates atomically, including simultaneous requests. A repeat signup returns “You’re already on the waitlist” and preserves the original signup date.

The Rust handler accepts JSON and regular HTML form submissions. The form works even when the browser WebAssembly client is unavailable. Requests over 2 KiB, including chunked requests, and cross-site browser submissions are rejected. Only the email and signup date are stored; email addresses are never logged. Signup does not send email; launch email delivery is a future task.

## Run

Requirements: Rust 1.95+, the `wasm32-unknown-unknown` target, Node.js, `worker-build` **0.8.7**, and `wasm-bindgen-cli` **0.2.129**. Topcoat is pinned to **0.6.2**, which works with the installed Rust 1.96 toolchain. Newer Topcoat versions require Rust 1.98. Cargo and npm lockfiles pin dependencies.

Install missing Rust tools:

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --version 0.8.7 --locked
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

Then:

```sh
cd apps/landing
npm ci
npm run build
npm run db:migrate:local
npm run dev
```

Open <http://localhost:8787>. For Pages, run `npm run db:migrate:pages` before `npm run dev:pages`, then open <http://localhost:8788>.

The build uses the SDK's generated bundler bridge with `--no-panic-recovery`; the SDK's experimental reset-state build mode was incompatible with this Wasm toolchain. No custom application Worker shim is used. Optional `WASM_BINDGEN_BIN` and `ESBUILD_BIN` environment variables let worker-build use already-installed binaries.

Workers and Pages use separate local database directories. Stop the relevant dev server before applying local migrations or using the D1 CLI against its local database. Rebuild after changing Rust. If Wrangler reports a filesystem watcher limit, restart the preview after rebuilding or changing assets.

In the Codex filesystem sandbox, Rust build tools were installed under `/private/tmp/relay-rust-tools`, with Cargo cache under `/private/tmp/relay-cargo`. To rebuild in that environment:

```sh
PATH=/private/tmp/relay-rust-tools/bin:$PATH \
CARGO_HOME=/private/tmp/relay-cargo \
WASM_BINDGEN_BIN=/private/tmp/relay-rust-tools/bin/wasm-bindgen \
ESBUILD_BIN=./node_modules/.bin/esbuild npm run build
```

## Deploy

The site is deployed on Cloudflare Workers at [relay.smbl.dev](https://relay.smbl.dev). Deployment settings live in an ignored `.env.deploy` file, outside the public assets. The shared Wrangler files contain only a local database placeholder and no production account, database ID, or domain.

To self-host, sign in to your Cloudflare account and create your database:

```sh
npx wrangler login
CLOUDFLARE_ACCOUNT_ID=your_account_id npx wrangler d1 create relay-waitlist --no-update-config
cp .env.example .env.deploy
```

Fill in `.env.deploy` with your `CLOUDFLARE_ACCOUNT_ID` and the returned `CLOUDFLARE_D1_DATABASE_ID`. Optionally set `RELAY_DOMAIN` to a hostname in a Cloudflare zone you own. Without a domain, deployment uses your `workers.dev` address. Change `name` in the shared Wrangler files if you want a different Worker or Pages project name.

The deployment helper is Rust. It reads `.env.deploy` or environment variables (environment variables take priority), requires both IDs, and writes a temporary Wrangler configuration under ignored `.wrangler/deploy/`. It passes the account through `CLOUDFLARE_ACCOUNT_ID` and supplies the D1 binding in the generated configuration. The deploy, dry-run, and remote migration commands stop before contacting Cloudflare if either ID is missing or invalid. Local previews need neither ID. In CI, set the two variables and a Cloudflare API token in the CI environment instead of copying the local file.

For later updates, sign in with `npx wrangler login` if needed, then:

```sh
npm run db:migrate:remote
npm run build
npm run check
npm run deploy
```

Alternatively, create the Pages project once using the same local settings, then deploy the advanced-mode output:

```sh
cargo run --locked --quiet --bin cloudflare -- pages pages project create relay-landing-spike --production-branch main
npm run deploy:pages
```

Use the npm deployment commands so the required environment settings are applied. Only generated public assets and the Rust Wasm Worker are uploaded. Product planning documents, deployment settings, and source stay outside asset directories.

## Verification

Verified locally with Wrangler **4.147.0**:

- Rust Worker and browser WebAssembly release builds succeed.
- Workers and Pages both return HTTP 200 with the same HTML as the static export.
- Workers deployment dry run passes: approximately **470 KiB total / 166 KiB gzip**, excluding separately uploaded browser assets.
- Three deployment configuration tests pass; missing account or D1 environment values stop both deployment paths before Wrangler runs. Git ignore checks exclude local settings and generated configs while keeping `.env.example` shareable.
- All seven Rust D1 integration tests pass on Workers and Pages: normalized duplicates, twelve simultaneous signups, invalid input, oversized and chunked bodies, origin/method/format guards, and normal HTML forms.
- Browser checks pass for mode switching, keyboard navigation, prompt selection, success and duplicate signup feedback, with no console errors.

With the corresponding dev server running:

```sh
npm run test:waitlist
WAITLIST_TEST_URL=http://localhost:8788 npm run test:waitlist
```

Tests use synthetic `example.com` emails and refuse non-local URLs. In the sandbox, add `CARGO_HOME=/private/tmp/relay-cargo` to those commands.

Public edge execution is verified: the HTTPS page and browser assets return 200, the page matches the Rust export, signup returns 201, normalized duplicates return 200, and twelve simultaneous signups create one record. Invalid email, cross-site submission, and unsupported method checks return 400, 403, and 405. Synthetic production test entries were removed after verification.

References: [Topcoat](https://github.com/tokio-rs/topcoat), [Cloudflare Rust SDK](https://github.com/cloudflare/workers-rs), [Rust on Workers](https://developers.cloudflare.com/workers/languages/rust/).
