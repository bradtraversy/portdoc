# PortDoc

Local dev server control panel. Single Rust binary that serves a web dashboard showing what dev apps are running, which project owns each port, and what URL to open.

Status: scaffold only. Product spec lives in the vault at `Projects/PortDoc/SPEC.md`.

## Layout

- `src/` - Rust binary: clap CLI entry, axum server
- `web/` - React + TypeScript + Vite frontend

## Commands

Backend:

```sh
cargo check          # typecheck
cargo run            # start server on 127.0.0.1:7788
cargo run -- --port 7799
```

Frontend:

```sh
cd web
npm install
npm run dev          # Vite dev server
npm run build        # production build to web/dist
npm run lint
```
