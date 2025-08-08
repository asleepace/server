# TODO

This document outlines the roadmap for this project, technical decisions, architecture, and next tasks.
Pleaes keep information technical, concise when possible, and organized.

## 1. Core correctness

- [x] Change default HTTP version to HTTP/1.1 (responses currently say HTTP/2.0)
- [x] Rename HttpRequest.event_souce → event_source
- [x] Remove unsafe from middleware chain; use immutable recursion or iterative execution
- [x] Store dynamic route parameters on HttpRequest; expose getters (req.param("key"))
- [x] Make public directory configurable via build mode; add PUBLIC_DIR env override
- [x] Fix Config.copy to preserve current port

## 2. Concurrency and load readiness (no external deps)

Short-term: keep sync code simple but ready to scale.

- [x] Prepare simple bounded threadpool in server accept loop (std-only, `--workers` flag)
- [x] Extract pool into reusable abstraction (`BoundedWorkerPool<T>`)
- [x] Handoff SSE connections to SSE manager after `event_source()` (Option 2)
- [x] Increase SSE keep-alive interval (default 15s); later make configurable
- [ ] Ensure request handling is side-effect free and concurrent-safe via SharedState (audit + tests)

## 3. Static files

- [ ] Consider mmap/caching and ETag/If-Modified-Since (later)

## 4. Routing

- [ ] Replace linear matcher with radix/trie (later)

## 5. CLI (keep homegrown for now)

- [ ] Add flags later: --workers, --public-dir, --sse-interval-ms

## 6. Future HTTP work (defer)

- [ ] Keep-alive/chunked responses for HTTP/1.1
- [ ] Optional TLS/ALPN, HTTP/2 and HTTP/3 via h2/quinn (future)

## 7. Observability/testing

- [ ] Add middleware timing, request logging levels, and basic tests for routes/middleware/static

## 8. Near-term tasks (this phase)

- [x] ResponseWriter abstraction that writes headers/body without closing socket
- [x] Keep-alive request loop with safe defaults (timeouts, single connection serves multiple requests)
- [x] Header/request/body limits to mitigate abuse (line length, total headers/bytes, content-length cap)
- [x] Basic static path resolution cache with short TTL to avoid repeated fs checks
- [x] SSE heartbeat interval configurable via `SSE_HEARTBEAT_SECS`
- [x] CLI flags (no deps): `--workers`, `--queue-capacity`, `--public-dir`, `--sse-heartbeat-secs`
