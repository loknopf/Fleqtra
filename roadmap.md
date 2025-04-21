# 📦 Object Store Roadmap

## ✅ MVP (Minimum Viable Product)
- [ ] Basic object upload & download API
- [ ] Store objects on local disk or pluggable backend (e.g., S3)
- [ ] PostgreSQL metadata storage
- [ ] User-defined tags (JSONB-based)
- [ ] Object querying via tags and metadata
- [ ] Simple object buckets/namespaces
- [ ] Versioning support per object
- [ ] Automatic `latest_version` tracking

---

## 🧠 Core Features (v1.x)
- [ ] Structured tag-based querying (SQL exposed via API)
- [ ] Relational object linking (e.g. derived_from, belongs_to)
- [ ] Data lifecycle policies
  - [ ] Global TTL or archive rules
  - [ ] Per-tag or per-object policies
- [ ] Version retention rules
  - [ ] Max version count per object
  - [ ] Max age for versions
  - [ ] Tag-based retention filters
- [ ] Virtual collections ("smart folders") based on tag filters
- [ ] JSONB metadata per object version (custom schemas)

---

## 🚀 Advanced Features (v2.x)
- [ ] Full-text search (filename, description, etc.)
- [ ] Policy-driven cold storage (move to glacier/tape/archive backend)
- [ ] Immutable object support (write-once)
- [ ] Pre-signed URLs for temporary access
- [ ] Access control system (API tokens, role-based permissions)
- [ ] Object deduplication (content-addressable storage)
- [ ] CLI and SDKs (Rust, Python, TypeScript)
- [ ] Object arrival workflows (ETL hooks, auto-thumbnailing, etc.)
- [ ] Audit log of changes per object

---

## 🖥️ Dashboard (optional, v3.x)
- [ ] Web-based UI to browse & query objects
- [ ] Lifecycle policy editor
- [ ] Version history viewer
- [ ] Usage stats & storage reports

---

## 🧪 Dev/Infra Quality of Life
- [ ] Docker-based deployment
- [ ] PostgreSQL migrations via CLI
- [ ] Metrics export (Prometheus, OpenTelemetry)
- [ ] Local dev mode with file backend

---

## 💡 Future Ideas
- [ ] Multi-tenant support
- [ ] Object stream ingestion (Kafka/NATS integration)
- [ ] Federation across clusters
- [ ] Data provenance tracking

