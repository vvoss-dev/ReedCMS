# Debug Tools

Development utilities for troubleshooting and inspection.

## Purpose

- **Request Inspector**: Complete request/response analysis
- **Cache Viewer**: ReedBase cache contents and statistics
- **Route Tester**: URL resolution validation
- **Config Inspector**: Configuration viewing

## Request Inspector

```bash
reed debug:request /knowledge
```

**Output**:
```
🔍 Request Inspector: /knowledge

URL Analysis:
  Route Match: knowledge (layout: knowledge, lang: en)

Headers:
  User-Agent: curl/7.88.1

Response Preview:
  Status: 200 OK
  Content-Type: text/html

Timing Breakdown:
  Routing: 2.1ms
  ReedBase: 8.3ms
  Rendering: 32.4ms
  Total: 42.8ms
```

## Cache Viewer

```bash
reed debug:cache
reed debug:cache text
reed debug:cache --search="knowledge"
```

**Output**:
```
📦 ReedBase Cache Contents

Text Cache (124 entries):
  knowledge.title (en): "Knowledge Base"
  blog.title (en): "Blog"

Cache Statistics:
  Total entries: 312
  Memory usage: ~1.2 MB
  Hit rate: 94.3%
```

## Route Tester

```bash
reed debug:route /knowledge
reed debug:route /de/wissen
```

**Output**:
```
🛣️  Route Test: /knowledge

Resolution:
  ✓ Match found
  Layout: knowledge
  Language: en

Template:
  ✓ Template exists

Assets:
  CSS: ✓
  JS: ✓
```

## Config Inspector

```bash
reed debug:config
```

Shows all configuration values from ReedBase.

## Performance

- Request inspection: < 50ms
- Cache viewing: < 100ms
- Route testing: < 10ms

## See README.md for complete implementation details.
