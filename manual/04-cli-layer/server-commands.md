# Server Commands

> Start, stop, and manage ReedCMS server

**Implementation:** REED-04-09  
**Status:** ✅ Complete  
**File:** `src/reedcms/cli/server_commands.rs`

---

## Overview

Server commands control the ReedCMS HTTP/socket server with daemon and interactive modes.

**Global patterns:** See [Common Patterns](common-patterns.md) for flags, output formats, error codes.

---

## Commands

### `reed server:io`

Start server in interactive mode (foreground).

```bash
reed server:io [--port <port>] [--socket <path>] [--workers <n>]
```

**Flags:**
- `--port` - HTTP port (default: from `.env` or 8333)
- `--socket` - Unix socket path (overrides port)
- `--workers` - Worker threads (default: CPU cores)

**Examples:**
```bash
reed server:io                    # Default settings
reed server:io --port 3000        # Custom port
reed server:io --workers 4        # 4 worker threads
reed server:io --socket /tmp/reed.sock  # Unix socket
```

**Binding:** Controlled by `ENVIRONMENT` in `.env`:
- `dev` → `127.0.0.1:8333` (localhost)
- `prod` → `/var/run/reed.sock` (Unix socket)

**Stop:** `Ctrl+C` or `kill <pid>`

**Performance:** Startup < 50ms

---

### `reed server:start`

Start server in background (daemon mode).

```bash
reed server:start [--environment <env>] [--port <port>] [--workers <n>]
```

**Flags:**
- `--environment` - Environment: `dev`, `prod` (default: `prod`)
- `--port` - HTTP port (dev mode only)
- `--workers` - Worker threads

**Examples:**
```bash
reed server:start                        # Production mode
reed server:start --environment dev      # Development mode
reed server:start --port 3000 --workers 8
```

**PID file:** `.reed/server.pid`

**Logs:** `.reed/logs/server.log`

**Auto-stop:** Stops existing instance before starting new one

---

### `reed server:stop`

Stop running server.

```bash
reed server:stop
```

**Example:**
```bash
reed server:stop
# Output: ✓ Server stopped (PID: 12345)
```

**Graceful shutdown:** Waits for active requests to complete (max 10s timeout)

**PID file:** Removed automatically

---

### `reed server:restart`

Restart server (stop + start).

```bash
reed server:restart [--environment <env>]
```

**Example:**
```bash
reed server:restart
reed server:restart --environment dev
```

**Equivalent to:**
```bash
reed server:stop
reed server:start --environment <env>
```

---

### `reed server:status`

Check server status.

```bash
reed server:status
```

**Output (running):**
```
✓ Server running
  PID:         12345
  Environment: prod
  Binding:     /var/run/reed.sock
  Workers:     8
  Uptime:      2h 15m
```

**Output (stopped):**
```
✗ Server not running
```

**Exit codes:**
- `0` - Server running
- `1` - Server stopped

---

### `reed server:logs`

View server logs.

```bash
reed server:logs [--lines <n>] [--follow]
```

**Flags:**
- `--lines` - Number of lines to show (default: 50)
- `--follow` - Tail logs in real-time (`-f`)

**Examples:**
```bash
reed server:logs                # Last 50 lines
reed server:logs --lines 100    # Last 100 lines
reed server:logs --follow       # Live tail (Ctrl+C to stop)
```

**Log file:** `.reed/logs/server.log`

**Format:**
```
2025-01-15T10:30:00Z [INFO]  Server started on /var/run/reed.sock
2025-01-15T10:30:15Z [INFO]  GET /knowledge → 200 (15ms)
2025-01-15T10:30:20Z [ERROR] POST /api/data → 500 (Database error)
```

---

## Server Configuration

### Environment-Based Binding

**Controlled by:** `ENVIRONMENT` in `.env`

**Development (`dev`):**
```env
ENVIRONMENT=dev
```
- Binds to: `127.0.0.1:8333`
- Hot-reload: Enabled
- Debug logs: Verbose

**Production (`prod`):**
```env
ENVIRONMENT=prod
```
- Binds to: `/var/run/reed.sock`
- Hot-reload: Disabled
- Debug logs: Errors only

### Server Settings

**File:** `.reed/server.csv`

**Settings:**
```
key|value|description
workers|8|Worker thread count
timeout|30|Request timeout (seconds)
enable_cors|false|CORS support
max_body_size|10485760|Max request body (10MB)
```

**Set via:**
```bash
reed server:set workers "16"
reed server:set timeout "60"
```

**See:** [Data Commands → Configuration Operations](data-commands.md#configuration-operations)

---

## Common Workflows

### Development Server

```bash
# Start in foreground with hot-reload
ENVIRONMENT=dev reed server:io

# Or background daemon
ENVIRONMENT=dev reed server:start

# View logs
reed server:logs --follow
```

### Production Server

```bash
# Start daemon (Unix socket)
ENVIRONMENT=prod reed server:start

# Check status
reed server:status

# View logs
reed server:logs --lines 100
```

### Configuration Update

```bash
# Update settings
reed server:set workers "16"

# Restart to apply
reed server:restart
```

### Debug Issues

```bash
# Check status
reed server:status

# View recent logs
reed server:logs --lines 200

# Restart server
reed server:restart

# Watch logs in real-time
reed server:logs --follow
```

---

## Process Management

### PID File

**Location:** `.reed/server.pid`

**Format:** Single line with process ID
```
12345
```

**Auto-managed:** Created on start, removed on stop

### Log Files

**Location:** `.reed/logs/`

**Files:**
- `server.log` - Main server log
- `access.log` - HTTP access log (optional)
- `error.log` - Error log (optional)

**Rotation:** Manual (add to cron if needed)

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| `io` startup | < 50ms | HTTP mode |
| `start` (daemon) | < 100ms | Background spawn |
| `stop` | < 10s | Graceful shutdown |
| `restart` | < 110ms | Stop + start |
| `status` | < 10ms | PID check |
| `logs` | < 50ms | File read |

**Worker threads:** Default = CPU cores, adjust with `--workers`

---

## Error Handling

### Port Already in Use

```bash
reed server:io --port 8333
# Error: Address already in use (port 8333)
```

**Solution:**
```bash
# Find process using port
lsof -i :8333

# Stop conflicting process
kill <pid>

# Or use different port
reed server:io --port 8334
```

### Socket Permission Denied

```bash
reed server:start  # prod mode
# Error: Permission denied (/var/run/reed.sock)
```

**Solution:**
```bash
# Run with sudo or adjust socket path
sudo reed server:start

# Or use custom socket
reed server:io --socket /tmp/reed.sock
```

### Server Won't Stop

```bash
reed server:stop
# Error: Server not responding
```

**Solution:**
```bash
# Force kill
cat .reed/server.pid | xargs kill -9

# Remove stale PID file
rm .reed/server.pid
```

---

## Best Practices

**Use daemon mode in production:**
```bash
# ✅ Good - runs in background
reed server:start

# ❌ Bad - blocks terminal
reed server:io
```

**Monitor logs:**
```bash
# ✅ Good - regular monitoring
reed server:logs --lines 100 | grep ERROR

# Set up log rotation
logrotate /etc/logrotate.d/reedcms
```

**Graceful restarts:**
```bash
# ✅ Good - graceful shutdown
reed server:restart

# ❌ Bad - force kill
kill -9 $(cat .reed/server.pid)
```

**Environment variables:**
```bash
# ✅ Good - explicit environment
ENVIRONMENT=prod reed server:start

# ❌ Bad - assumes environment
reed server:start
```

---

## System Integration

### systemd Service

Create `/etc/systemd/system/reedcms.service`:

```ini
[Unit]
Description=ReedCMS Server
After=network.target

[Service]
Type=forking
User=www-data
Group=www-data
WorkingDirectory=/var/www/reedcms
Environment="ENVIRONMENT=prod"
ExecStart=/usr/local/bin/reed server:start
ExecStop=/usr/local/bin/reed server:stop
PIDFile=/var/www/reedcms/.reed/server.pid

[Install]
WantedBy=multi-user.target
```

**Commands:**
```bash
systemctl enable reedcms
systemctl start reedcms
systemctl status reedcms
```

### Nginx Proxy

Proxy HTTP requests to Unix socket:

```nginx
upstream reedcms {
    server unix:/var/run/reed.sock;
}

server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://reedcms;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

**See also:**
- [Common Patterns](common-patterns.md) - Global flags, errors
- [Config Commands](config-commands.md) - Server configuration
- [Server Layer](../06-server-layer/) - Server implementation details
