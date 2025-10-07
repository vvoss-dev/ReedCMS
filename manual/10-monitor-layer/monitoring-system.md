# Monitoring System

FreeBSD-style syslog with metrics collection for production observability.

## Purpose

- **RFC 5424 Compliance**: Standard syslog format
- **Metrics Collection**: Request, ReedBase, template, system metrics
- **Health Checks**: `/health` and `/metrics` endpoints
- **Log Management**: Rotation, compression, retention

## FreeBSD Syslog Format

```
{timestamp} {hostname} {process}[{pid}]: {level}: {message}

Examples:
Dec 15 14:23:01 server reedcms[1234]: INFO: Server started on 127.0.0.1:3000
Dec 15 14:23:02 server reedcms[1234]: INFO: METRIC[counter] requests_total: 42
Dec 15 14:23:03 server reedcms[1234]: ERROR: Connection failed: timeout
```

## Log Levels (RFC 5424)

| Level | Value | Description | Use Case |
|-------|-------|-------------|----------|
| EMERG | 0 | Emergency | System unusable |
| ALERT | 1 | Alert | Immediate action required |
| CRIT | 2 | Critical | Critical conditions |
| ERROR | 3 | Error | Error conditions |
| WARN | 4 | Warning | Warning conditions |
| NOTICE | 5 | Notice | Normal but significant |
| INFO | 6 | Informational | Normal operations |
| DEBUG | 7 | Debug | Development only |

## Output Modes

**Silent**: Metrics only, no logging (production default)
**Log**: Write to `.reed/flow/reedmonitor.log`
**Forward**: Forward to system syslog/journald
**Both**: Log file + forward to system

## Metrics Collected

### Request Metrics
- Total count, response times (min/max/avg/p95/p99)
- Status code distribution, throughput

### ReedBase Metrics
- Cache hit rate, lookup times, total lookups

### Template Metrics
- Render times, cache hits, errors

### System Metrics
- Memory usage (RSS), CPU, connections, uptime

## See README.md for complete implementation details and CLI commands.
