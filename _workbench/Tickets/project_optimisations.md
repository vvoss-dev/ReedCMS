# ReedCMS - Architectural Decisions

**Current State**: 2025-10-06  
**Active Decisions**: 50

---

## Decision Registry

```csv
ID,Decision,Rationale,Date,Status
D001,CSV-based storage,"Fast, git-friendly, direct editing",2025-01-15,Active
D002,Rust implementation,"Memory safety, performance, type safety",2025-01-15,Active
D003,Matrix CSV for complex data,"Enhanced relationships, backward compatible",2025-01-15,Active
D004,ReedStream universal interface,"Consistent cross-module communication",2025-01-15,Active
D005,Environment-aware fallback,"Flexible deployment, inheritance patterns",2025-01-15,Active
D006,FreeBSD syslog format,"Professional system integration, standard compliance",2025-01-15,Active
D007,XZ backup compression,"Efficient storage (~10x ratio), automatic recovery",2025-01-15,Active
D008,Argon2 password hashing,"Security best practice, future-proof",2025-01-15,Active
D009,Unix socket deployment,"High performance, security, nginx integration",2025-01-15,Active
D010,Template hot-reload,"Development efficiency, immediate feedback",2025-01-15,Active
D011,Rate limiting system,"API protection, DoS prevention",2025-01-15,Active
D012,Hierarchical taxonomy,"Universal categorisation, scalable organisation",2025-01-15,Active
D013,Permission caching,"Sub-millisecond lookups, performance optimisation",2025-01-15,Active
D014,CLI command bridge,"Zero business logic duplication, direct execution",2025-01-15,Active
D015,Flow persistence rules,"Clear data ownership, service coordination",2025-01-15,Active
D016,Component inclusion functions,"KISS principle, automatic variant resolution",2025-01-30,Active
D017,Taxonomy-based navigation,"Drupal-style flexibility, multiple menu locations",2025-01-30,Active
D018,Session hash asset bundling,"MD5-based cache-busting, on-demand generation",2025-01-30,Active
D019,Client detection via cookie,"Server-side responsive rendering, no JS needed",2025-01-30,Active
D020,SVG icon molecule wrapper,"Atomic Design compliance, accessibility support",2025-01-30,Active
D021,Full namespace text keys,"No auto-prefixing, explicit key format validation",2025-01-30,Active
D022,RESTful API architecture,"Resource-based endpoints, standard HTTP methods, JSON responses",2025-02-01,Active
D023,Direct CSV fallback for API,"Immediate functionality without ReedBase cache dependency",2025-02-01,Active
D024,Security middleware layering,"AuthMiddleware before SecurityMiddleware, cascading checks",2025-02-01,Active
D025,SHA-256 for API keys,"Fast hashing for keys, Argon2 reserved for passwords only",2025-02-01,Active
D026,Sliding window rate limiting,"Per-user per-operation tracking, more accurate than fixed windows",2025-02-01,Active
D027,Code reuse enforcement,"Mandatory function registry check before writing new code",2025-02-01,Active
D028,MD5 session hash strategy,"8-character hash over all CSS/JS for cache-busting and versioning",2025-02-04,Active
D029,On-demand CSS bundling,"Generate bundles on first request, not at build time",2025-02-04,Active
D030,Component discovery from templates,"Automatic Jinja parsing for organism/molecule/atom dependencies",2025-02-04,Active
D031,CSS minification without tools,"Custom minifier for 60-70% reduction, no external dependencies",2025-02-04,Active
D032,Source map v3 generation,"Browser DevTools debugging support for minified CSS",2025-02-04,Active
D033,ES6 and CommonJS support,"Dual format support for maximum compatibility in dependency resolution",2025-02-04,Active
D034,Topological module sorting,"Dependencies loaded before dependents, prevents undefined reference errors",2025-02-04,Active
D035,IIFE module wrapping,"Prevents global scope pollution, maintains module isolation",2025-02-04,Active
D036,Simplified tree shaking,"Remove unused exports without full AST parsing for performance",2025-02-04,Active
D037,Variant-independent JS,"Single JS bundle per layout works across mouse/touch/reader variants",2025-02-04,Active
D038,ETag generation from metadata,"mtime+size hash instead of content hash, O(1) performance, no file reading",2025-02-04,Active
D039,Pre-compression at build time,"Gzip and Brotli pre-compressed, zero runtime CPU overhead for compression",2025-02-04,Active
D040,Compression method priority,"Brotli > Gzip > None based on Accept-Encoding header",2025-02-04,Active
D041,Long-lived cache headers,"CSS/JS: 1 year immutable, Images: 30 days, leveraging session hash versioning",2025-02-04,Active
D042,Path traversal prevention,"Canonicalization-based security check before serving any file",2025-02-04,Active
D043,Security headers standard,"X-Content-Type-Options: nosniff, X-Frame-Options: DENY on all static assets",2025-02-04,Active
D044,Dot-notation for all CSV keys,"Lowercase dot-separated keys (e.g., landing.hero.title), no hyphens or underscores",2025-02-04,Active
D045,LTO and symbol stripping,"Link-time optimization for 15MB→6MB binary reduction",2025-02-04,Active
D046,UPX binary compression,"Further compression to ~3MB, still fast startup",2025-02-04,Active
D047,Incremental rebuild detection,"300ms debouncing, rebuild only changed layouts/components",2025-02-04,Active
D048,Language-filtered route lookup,"Route lookup filters by language parameter to prevent cross-language matching",2025-10-06,Active
D049,Path-segment-only routes.csv,"Store only path segments (wissen, knowledge) without language prefixes in routes.csv",2025-10-06,Active
D050,Root URL language redirect,"/ redirects to /de/ or /en/ based on Accept-Language header with 301 for SEO",2025-10-06,Active
D051,CSV as runtime truth,"CSV files are single source of truth at runtime, Reed.toml only for bootstrap",2025-10-06,Active
D052,Bidirectional TOML-CSV sync,"config:sync (TOML→CSV) and config:export (CSV→TOML) for version control",2025-10-06,Active
D053,Environment-based server binding,"ENVIRONMENT variable in .env controls dev (port) vs prod (socket) binding",2025-10-06,Active
D054,Destructive sync protection,"config:sync requires --force flag and shows warning to prevent accidental overwrites",2025-10-06,Active
D055,Template engine legacy pattern,"Base singleton environment + per-request clone + language-specific filters",2025-10-07,Active
D056,Per-request filter registration,"Clone environment and add text/route filters with request language for correct routing",2025-10-07,Active
D057,Avoid .local TLD for development,"Use .dev or other TLD to prevent mDNS 5-second timeout on macOS",2025-10-07,Active
```

---

## Layer Completion Status

```
✅ REED-01: Foundation Layer     2/2  (100%)
✅ REED-02: Data Layer            6/6  (100%)
✅ REED-03: Security Layer        2/2  (100%)
🚧 REED-04: CLI Layer             8/9  ( 89%)  ← REED-04-12 Complete
✅ REED-05: Template Layer        3/3  (100%)
🚧 REED-06: Server Layer          4/5  ( 80%)
✅ REED-07: API Layer             2/2  (100%)
✅ REED-08: Asset Layer           3/3  (100%)
✅ REED-09: Build Layer           3/3  (100%)
⏳ REED-10: Monitor Layer         0/4  (  0%)
⏳ REED-11: Extension Layer       0/4  (  0%)
```

**Overall Progress**: 33/43 tickets complete (77%)

---

**Last Updated**: 2025-10-06  
**Maintained By**: Vivian Voss <ask@vvoss.dev>
