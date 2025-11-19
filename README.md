# ServerOS - Rust Web Server Framework

A high-performance, secure web server framework built in Rust with dynamic route matching, middleware composition, and enterprise-grade security features.

## 🚀 Quick Start

```bash
# Development
cargo run

# Production build
cargo build --release
./target/release/server --host "0.0.0.0" --port 8080

# With custom configuration
./target/release/server --port 9000 --host "localhost"
```

**Default**: `http://localhost:8080/`

## 🏗️ Architecture

### Request Flow

```
Client → Incoming Request → Middleware Chain → Route Handlers → Static File Middleware → Response
```

### Core Components

#### **Middleware System**

- **Response State Tracking**: Prevents duplicate responses
- **Composable Middleware**: Chain multiple middleware components
- **Error Boundaries**: Graceful error handling without panics

#### **Dynamic Route Matching**

```rust
// Static routes
server.route("/", |req| { /* handler */ });
server.route("/api/users", |req| { /* handler */ });

// Dynamic routes with parameters
server.route("/users/[userId]", |req| { /* handler */ });
server.route("/posts/[postId]/comments/[commentId]", |req| { /* handler */ });
```

#### **Static File Serving**

#### **Concurrency Model**

- Decision: bounded worker pool using std-only (Option B)

  - N worker threads (default: available_parallelism) pull `TcpStream`s from a bounded queue
  - Backpressure via queue capacity to prevent unbounded memory growth
  - Simple, dependency-free, predictable under load; suitable for massive parallel short-lived requests
  - Flag: `--workers N` to override worker count

  - SSE handoff (Option 2 atop pool): after upgrading a connection to SSE, the server hands the request to the SSE manager and frees the worker. Long-lived streams are maintained by `HttpConnections`, keeping the pool available for short-lived requests.

- **Secure Path Resolution**: Prevents path traversal attacks
- **Fallback Chain**: `file.html` → `folder/index.html` → `404.html`
- **Optional Middleware**: Only serves files if no response sent

## 🔒 Security Features

### Path Traversal Protection

```rust
// Blocked attacks
"/../../../etc/passwd"  // ❌ Blocked
"/~/.ssh/id_rsa"        // ❌ Blocked
"C:\\Windows\\System32" // ❌ Blocked
```

### Input Validation

- **Character Allowlist**: `a-zA-Z0-9`, `-`, `_`, `.`, `/`
- **Path Length Limits**: Maximum 255 characters
- **Boundary Enforcement**: All paths stay within `./src/public/`

### Security Logging

```bash
[security] Blocked request for: /../../../etc/passwd
[security] Path bounds violation for root path
```

## 📁 File Structure

```
server/
├── src/
│   ├── core/
│   │   ├── http/            # HTTP request/response handling
│   │   ├── middleware/      # Middleware system
│   │   ├── routes/          # Route matching & handlers
│   │   ├── security/        # Path validation & sanitization
│   │   └── server.rs        # Main server implementation
│   ├── public/              # Static assets
│   └── main.rs              # Application entry point
├── Cargo.toml
└── README.md
```

## 🛠️ Development

## 🌐 Client UI (Public Assets)

The browser UI lives in `src/public/` and is served statically by the server. It includes a minimal component system and an SSE-powered live session page.

### Entry points

- `src/public/session.html` loads the client via ESM:
  ```html
  <script type="module" src="/client.js"></script>
  ```
- `src/public/client.js` (ESM entry):
  - Imports a single component registry: `src/public/scripts/components/index.js`
  - Boots the session page (`bootstrapSessionPage`)
  - Applies a diagnostics flag to mute logs in production

### Components

- Base utilities and definitions: `src/public/components/index.js`
- Function-first components (export default fn) in `src/public/components/`:
  - `navbar.js` → `<cd-nav>`
  - `tabs.js` → `<cd-tabs>`
  - `snippet.js` → `<cd-snippet>`
- Central registry registers all: `src/public/scripts/components/index.js`

#### Linking step (inline events)

- Component factories may return a string or `{ html, methods }`.
  - `html`: component markup string
  - `methods`: map of handlers to bind
- Registry scans shadowRoot after render and links inline events written as `on*="@handlerName"`.

Example:

```js
export default function MyButton() {
  function showAlert() {
    alert("Hi!");
  }
  return {
    html: `<button onclick="@showAlert">Click</button>`,
    methods: { showAlert },
  };
}
```

### Live Event Stream (SSE)

- Shared helpers: `src/public/scripts/events.js`
  - `watchEvents({ eventSource, targetElement, onErrorDisconnect, maxLines, hotReload, hotReloadEventName })`
  - `parseEvent(event)` converts `data` and `event` to display text
  - `isNearBottom(container, threshold)` scroll behavior helper
- Session bootstrap: `src/public/scripts/session.js`
  - Reads session key from `window.location.pathname`
  - Starts SSE: `/events?s=<sessionKey>` targeting `#event-stream`
  - Persists sidebar open/width/theme in localStorage

Minimal usage example:

```html
<pre><code id="event-stream"></code></pre>
<script type="module">
  import { watchEvents } from '/scripts/events.js'
  watchEvents({ eventSource: '/events', targetElement: 'event-stream', maxLines: 5000 })
<\/script>
```

### Hot Reload via SSE

- `watchEvents` can auto-reload on a custom SSE event. Enabled in the session page by default.
- Options:
  - `hotReload: true`
  - `hotReloadEventName: 'hot-reload'` (default)
- Expected SSE frame (server-sent):

  ```
  event: hot-reload
  data: 1

  ```

- Behavior: cache-busts CSS `<link rel="stylesheet">` tags, then calls `location.reload()` (debounced).

### Client Commands via SSE

- The server can push ad-hoc client commands over SSE using event `client-cmd`.
- Default behavior: message is appended in the stream with a `[client]` prefix unless a custom handler is provided.
- Shorthand: POST `@client:reload` to trigger a hot reload.

Trigger examples:

```bash
# Hot reload all sessions
curl -X POST http://localhost:8080/__reload

# Hot reload a specific session
curl -X POST "http://localhost:8080/__reload?s=ABC123"

# Generic client message (shown in the stream by default)
curl -X POST http://localhost:8080/__client -d 'Hello from server!'

# Shorthand for reload via client command
curl -X POST http://localhost:8080/__client -d '@client:reload'
```

### Diagnostics (Client Logging)

- By default, logs are enabled on localhost and muted in production.
- Override at runtime in DevTools:
  ```js
  // Enable
  localStorage.setItem("diagnostics", "1");
  location.reload();
  // Disable
  localStorage.removeItem("diagnostics");
  location.reload();
  ```

### Client Tests

- Open `http://localhost:8080/test/` to run simple browser tests.
- Tests live in `src/public/test/index.html` and validate:
  - `isNearBottom`
  - `parseEvent`
  - `watchEvents` DOM updates and hot-reload handling (with mocked `EventSource`)

### Adding Routes

```rust
// Static route
server.route("/api/health", |req| {
    Ok(200) // Return status code
});

// Dynamic route with parameters
server.route("/api/users/[userId]", |req| {
    // TODO: Access userId parameter
    req.send_file("user.html")
});

// File serving route
server.route("/download/[filename]", |req| {
    req.send_file(&format!("files/{}", filename))
});
```

### Custom Middleware

```rust
// Authentication middleware
server.middleware(|req, next| {
    if !req.headers.has_auth_token() {
        return Ok(401);
    }
    next(req)
});

// Logging middleware
server.middleware(|req, next| {
    println!("{} {}", req.headers.method, req.uri);
    let result = next(req);
    println!("Response: {:?}", result);
    result
});
```

### Response State Management

```rust
// Check if response already sent
if req.is_response_sent() {
    return next(req);
}

// Mark response as sent
req.mark_response_sent(200);

// Handle errors
req.mark_error(std::io::Error::new(ErrorKind::NotFound, "Not found"));
```

## 🔍 Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Common Debug Points

```bash
# Middleware registration
[middleware] registering middleware...

# Route registration
[routes] registering route: /users/[userId] -> handler__users_userId

# Request handling
[http_request] new request: /users/123
[main] serving dynamic route: /users/[userId]
[server] finished with code: 200

# Security events
[security] Blocked request for: /../../../etc/passwd
```

### Error Handling

- **No Panics**: All errors handled gracefully
- **Error Logging**: Comprehensive error tracking
- **Status Codes**: Proper HTTP status code responses

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test security
cargo test routes
```

### Integration Tests

```bash
# Start server
cargo run &

# Test routes
curl http://localhost:8080/
curl http://localhost:8080/users/123
curl http://localhost:8080/posts/456/comments/789

# Test security
curl http://localhost:8080/../../../etc/passwd  # Should be blocked
```

## 📊 Performance

### Benchmarks

- **Request Processing**: < 1ms per request
- **Memory Usage**: ~2MB baseline
- **Concurrent Connections**: 1000+ simultaneous
- **Static File Serving**: Optimized with secure caching

### Optimization Tips

```rust
// Use Arc for shared state
let shared_data = Arc::new(Mutex::new(Data::new()));

// Minimize allocations in hot paths
let response = format!("{}", data); // Single allocation

// Use efficient data structures
let routes = HashMap::new(); // O(1) lookup
```

## 🔧 Configuration

### Environment Variables

```bash
RUST_LOG=debug          # Logging level
RUST_BACKTRACE=1        # Stack traces
```

### Command Line Options

```bash
--host "localhost"       # Bind address
--port 8080             # Port number
```

## 🚨 Security Considerations

### Path Traversal Protection

- **Input Sanitization**: All paths validated before use
- **Boundary Checking**: Paths cannot escape public directory
- **Character Filtering**: Only safe characters allowed

### Error Information Disclosure

- **Generic Errors**: No sensitive information in error messages
- **Security Logging**: All blocked requests logged
- **Graceful Degradation**: Server continues running after errors

### Static File Security

- **MIME Type Detection**: Proper content-type headers
- **File Size Limits**: Prevents memory exhaustion
- **Directory Listing**: Disabled by default

## 📈 Monitoring

### Health Checks

```bash
curl http://localhost:8080/health
```

### Metrics

- Request count per route
- Response time distribution
- Error rate tracking
- Security event monitoring

## 🤝 Contributing

### Code Style

- Follow Rust conventions
- Add tests for new features
- Update documentation
- Security review for new routes

### Testing Checklist

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Security tests pass
- [ ] Performance benchmarks
- [ ] Documentation updated

## 📄 License

MIT License - see LICENSE file for details.

---

**ServerOS**: Enterprise-grade web server framework with security-first design and flexible middleware architecture.
