# ZetaPrint Development Tasks

## Completed
- [x] Initial SvelteKit + Tailwind setup (Tailwind removed as per request)
- [x] Integrate standard CSS with Inter and JetBrains Mono fonts
- [x] Dashboard UI implementation (Mock Data, Real-time Printers Table, Health Status)
- [x] Convert app layout to use a collapsible left sidebar
- [x] Remove "Filter Total: Today" and "Search resources" from dashboard UI
- [x] Relocate `Logout` button to a profile dropdown in the top right header
- [x] Fix TypeScript type definitions (`@types/node` installed)

## Current: Backend SSE Notifications
- [x] Plan SSE Architecture
- [x] Add `tokio-stream` and `futures-util` to workspace `Cargo.toml`
- [x] Add dependencies to `core_api` `Cargo.toml`
- [x] Implement `AppState` with `broadcast::channel`
- [x] Create `handle_sse` endpoint in `/api/v1/dashboard/events`
- [/] Run `cargo check` to verify backend compilation (running)

## Current: Security & Auth Fixes
- [x] Migrate Authentication to HttpOnly Cookie (Sesuai KI)
  - [x] Set Cookie on login endpoint
  - [x] Middleware supports Cookie & Bearer

## Completed: Printer Management
- [x] `core_db`: CRUD operations for printers
- [x] `core_api`: POST/PUT/DELETE routes for printers
- [x] `frontend`: Build `/dashboard/printers` UI and logics

## Completed: Real-time DB & UI Integration
- [x] Wire up `core_db` queries (Stats, Printers, Queue)
- [x] Connect SvelteKit frontend to the new real API endpoints
- [x] Build UI for `/dashboard/queue` to fix 404
- [x] Build UI for `/dashboard/logs` to fix 404

## Completed: core_ipp
- [x] Pengembangan `core_ipp` (Print Server)
  - [x] Konversi ke server Axum (Port 631)
  - [x] Parsing request IPP (`ipp` crate v6)
  - [x] Integrasi evaluasi biaya (`core_accounting`), database, dan dispatch
