# Altaïr Groups Microservice

> **Collaborative learning groups with member management and lab/starpath assignments**
> 

[![Cloud Run](https://img.shields.io/badge/deploy-Cloud%20Run-blue)](https://cloud.google.com/run)

[![Rust](https://img.shields.io/badge/rust-nightly-orange)](https://www.rust-lang.org)

[![PostgreSQL](336791)](https://www.postgresql.org)

---

## Description

The **Altaïr Groups Microservice** manages collaborative learning groups where students can be organized by teachers or self-organized. It handles group creation, member management, and assignment of labs and starpaths to groups.

This service provides CRUD operations for groups, member management with role-based access, and assignment tracking for learning content.

**Key capabilities:**

- Create and manage learning groups (name, description)
- Add and remove members with role assignments
- Assign labs to groups for collective learning
- Assign starpaths to groups for structured pathways
- Track member roles (owner, admin, teacher, member)
- List group memberships and assignments

---

## ⚠️ Security Notice

**This service is currently in MVP stage with NO AUTHORIZATION.**

- ❌ **No auth validation** – All endpoints are publicly accessible
- ❌ **No ownership checks** – Anyone can modify any group
- ❌ **No role enforcement** – Role field exists but is not validated
- ❌ **Creator not auto-added** – Group creator is not automatically made a member

**Deployment requirement:** Must be migrated to Gateway-based authorization before production.

---

## Architecture

```
┌─────────────┐       ┌──────────────┐       ┌────────────────┐
│  Frontend   │──────▶│   Gateway    │──────▶│   Groups MS    │
│             │       │   (TODO)     │       │    (:3006)     │
└─────────────┘       └──────────────┘       └────────┬───────┘
                                                       │
                                                       ▼
                                               ┌───────────────┐
                                               │  PostgreSQL   │
                                               │   (Groups)    │
                                               └───────────────┘
                                                 groups
                                                 group_members
                                                 group_labs
                                                 group_starpaths
```

### Service Flow

1. **Creator creates group** → Defines name, description
2. **Teacher adds members** → Assigns users to group with roles
3. **Teacher assigns content** → Adds labs or starpaths to group
4. **Members access content** → View assigned labs/starpaths
5. **Teacher removes content** → Unassigns labs/starpaths as needed

---

## Tech Stack

| Component | Technology | Purpose |
| --- | --- | --- |
| **Language** | Rust (nightly) | High-performance async runtime |
| **HTTP Framework** | Axum 0.7 | HTTP routing and middleware |
| **Async Runtime** | Tokio | Async I/O and concurrency |
| **Database** | PostgreSQL | Group and member persistence |
| **DB Client** | SQLx 0.8 | Compile-time checked queries |
| **Logging** | tracing + EnvFilter | Structured logging |
| **CI/CD** | GitHub Actions | fmt, clippy, tests |
| **Deployment** | Google Cloud Run | Serverless auto-scaling |

---

## Requirements

### Development

- **Rust** nightly toolchain
- **Docker** & Docker Compose
- **PostgreSQL** 14+ (via `docker compose up postgres`)

### Production (Cloud Run)

- **DATABASE_URL** environment variable (PostgreSQL connection string)
- **PORT** environment variable (default: `3006`)

### Environment Variables

```bash
# Database (required)
DATABASE_URL=postgresql://altair:altair@localhost:5434/altair_groups_db

# Server configuration
PORT=3006                                       # Server port (default: 3006)
RUST_LOG=info                                   # Log level filter

# Gateway (required)
GATEWAY_URL=http://localhost:3000
```

**⚠️ Database Port Note:** If using `altair-infra` Docker Compose, the Groups database is on port `5434`, not the default `5432`.

---

## Installation

### 0. Start infrastructure (database required)

```bash
cd ../altair-infra
docker compose up postgres
```

### 1. Build the Docker image

```bash
cd altair-groups-ms
docker build -t altair-groups-ms .
```

### 2. Run the service

```bash
docker run --rm -it \
  --network altair-infra_default \
  -p 3006:3006 \
  --env-file .env \
  --name altair-groups-ms \
  altair-groups-ms
```

**Note:** The service is designed to be destroyed when the terminal closes. Rebuild is necessary for code changes.

---

## Usage

### API Endpoints

#### **GET /health**

Health check for liveness/readiness probes.

**Response:**

```json
{
  "success": true,
  "data": null,
  "meta": {
    "request_id": "...",
    "timestamp": "2026-02-08T18:00:00Z"
  }
}
```

---

#### **GET /groups**

List all groups (ordered by creation date, descending).

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "group_id": "550e8400-e29b-41d4-a716-446655440000",
      "creator_id": "...",
      "name": "ISEN Cybersecurity",
      "description": "Student group for cybersecurity courses",
      "created_by": "...",
      "created_at": "2026-02-08T18:00:00Z"
    }
  ]
}
```

---

#### **GET /groups/:id**

Get group details by ID.

**Response:**

```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "creator_id": "...",
    "name": "ISEN Cybersecurity",
    "description": "...",
    "created_by": "...",
    "created_at": "..."
  }
}
```

**Error (404):**

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Group not found"
  }
}
```

---

#### **POST /groups**

Create a new group.

**Request:**

```json
{
  "name": "ISEN Cybersecurity",
  "description": "Student group for cybersecurity courses",
  "creator_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_by": "550e8400-e29b-41d4-a716-446655440000"
}
```

**⚠️ Security Issue:** `creator_id` and `created_by` are accepted from request body (spoofable). Should be extracted from Gateway headers.

**⚠️ Known Issue:** Group creator is NOT automatically added as a member. Must manually add via `/groups/:id/members`.

**Response:**

```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "creator_id": "...",
    "name": "ISEN Cybersecurity",
    "description": "...",
    "created_by": "...",
    "created_at": "2026-02-08T18:00:00Z"
  }
}
```

---

#### **PUT /groups/:id**

Update a group.

**Request:**

```json
{
  "name": "ISEN Advanced Cybersecurity",
  "description": "Updated description"
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "name": "ISEN Advanced Cybersecurity",
    "description": "Updated description",
    "created_at": "..."
  }
}
```

---

#### **DELETE /groups/:id**

Delete a group.

**Response:**

```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

---

#### **GET /groups/:id/members**

List all members in a group (ordered by join date).

**Response:**

```json
{
  "success": true,
  "data": [
    {
      "group_id": "...",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "role": "owner",
      "joined_at": "2026-02-08T18:00:00Z"
    },
    {
      "group_id": "...",
      "user_id": "...",
      "role": "member",
      "joined_at": "2026-02-08T18:05:00Z"
    }
  ]
}
```

**Roles:** `owner`, `admin`, `teacher`, `member`

---

#### **POST /groups/:id/members**

Add a member to a group.

**Request:**

```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Behavior:**

- Idempotent: Uses `INSERT ... ON CONFLICT DO NOTHING`
- Always assigns role `"member"` (cannot set owner/admin/teacher via API)
- Returns success even if user already exists

**Response:**

```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "user_id": "...",
    "role": "member",
    "joined_at": "2026-02-08T18:10:00Z"
  }
}
```

**⚠️ Limitation:** Cannot specify role when adding member. Always defaults to `"member"`.

---

#### **DELETE /groups/:id/members/:user_id**

Remove a member from a group.

**Response:**

```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

**Error (404):**

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Member not found in group"
  }
}
```

---

#### **GET /groups/:id/labs**

List all labs assigned to a group.

**Response:**

```json
{
  "success": true,
  "data": [
    "550e8400-e29b-41d4-a716-446655440000",
    "660e8400-e29b-41d4-a716-446655440001"
  ]
}
```

**⚠️ Note:** Returns only lab UUIDs. Metadata like `assigned_at` and `due_date` are in the database but not exposed via API.

---

#### **POST /groups/:id/labs**

Assign a lab to a group.

**Request:**

```json
{
  "lab_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Behavior:**

- Idempotent: Uses `INSERT ... ON CONFLICT DO NOTHING`
- Returns success even if lab already assigned

**Response:**

```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "lab_id": "..."
  }
}
```

---

#### **DELETE /groups/:id/labs/:lab_id**

Unassign a lab from a group.

**Response:**

```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

---

#### **GET /groups/:id/starpaths**

List all starpaths assigned to a group.

**Response:**

```json
{
  "success": true,
  "data": [
    "550e8400-e29b-41d4-a716-446655440000"
  ]
}
```

**⚠️ Note:** Returns only starpath UUIDs. `assigned_at` is in the database but not exposed.

---

#### **POST /groups/:id/starpaths**

Assign a starpath to a group.

**Request:**

```json
{
  "starpath_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Behavior:**

- Idempotent: Uses `INSERT ... ON CONFLICT DO NOTHING`

**Response:**

```json
{
  "success": true,
  "data": {
    "group_id": "...",
    "starpath_id": "..."
  }
}
```

---

#### **DELETE /groups/:id/starpaths/:starpath_id**

Unassign a starpath from a group.

**Response:**

```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

---

## Database Schema

### `groups` Table

| Column | Type | Constraints | Description |
| --- | --- | --- | --- |
| `group_id` | UUID | PRIMARY KEY | Group identifier |
| `creator_id` | UUID | NOT NULL | User who created the group |
| `name` | TEXT | NOT NULL | Group name |
| `description` | TEXT | NULLABLE | Group description |
| `created_by` | UUID | NOT NULL | User who created (duplicate of creator_id) |
| `created_at` | TIMESTAMP | NOT NULL | Creation timestamp |

---

### `group_members` Table

| Column | Type | Constraints | Description |
| --- | --- | --- | --- |
| `group_id` | UUID | NOT NULL | Group identifier |
| `user_id` | UUID | NOT NULL | Member user identifier |
| `role` | TEXT | NOT NULL | Member role (owner/admin/teacher/member) |
| `joined_at` | TIMESTAMP | NOT NULL | Join timestamp |

**Constraints:**

- `(group_id, user_id)` – UNIQUE (no duplicate members)

---

### `group_labs` Table

| Column | Type | Constraints | Description |
| --- | --- | --- | --- |
| `group_id` | UUID | NOT NULL | Group identifier |
| `lab_id` | UUID | NOT NULL | Lab identifier |
| `assigned_at` | TIMESTAMP | NULLABLE | Assignment timestamp (not exposed) |
| `due_date` | TIMESTAMP | NULLABLE | Due date (not exposed) |

**Constraints:**

- `(group_id, lab_id)` – UNIQUE (no duplicate assignments)

---

### `group_starpaths` Table

| Column | Type | Constraints | Description |
| --- | --- | --- | --- |
| `group_id` | UUID | NOT NULL | Group identifier |
| `starpath_id` | UUID | NOT NULL | Starpath identifier |
| `assigned_at` | TIMESTAMP | NULLABLE | Assignment timestamp (not exposed) |

**Constraints:**

- `(group_id, starpath_id)` – UNIQUE (no duplicate assignments)

---

## Project Structure

```
altair-groups-ms/
├── Cargo.toml                    # Rust dependencies
├── Dockerfile                    # Multi-stage build
├── .env                          # Environment variables
└── src/
    ├── main.rs                  # Server bootstrap, CORS, routes
    ├── state.rs                 # AppState (DB pool + service)
    ├── error.rs                 # AppError type
    ├── routes/
    │   ├── mod.rs              # Route declarations
    │   ├── health.rs           # Health check endpoint
    │   └── groups.rs           # All group endpoints
    ├── services/
    │   └── groups_service.rs   # Core group logic
    └── models/
        ├── group.rs            # Group data models
        ├── member.rs           # Member and role models
        ├── assignments.rs      # Lab/starpath assignment models
        └── api.rs              # API response wrappers
```

---

## Deployment (Google Cloud Run)

The service is containerized and deployed to **Google Cloud Run** as an internal service.

### Container Configuration

- Listens on port `3006` (configurable via `PORT` env variable)
- Multi-stage Docker build optimizes image size
- Rust nightly toolchain for compilation

**⚠️ Note:** Current code has hardcoded `bind("0.0.0.0:3006")` which ignores `PORT` env variable.

### Runtime Requirements

- `DATABASE_URL` environment variable (Cloud SQL or external PostgreSQL)
- Must be deployed in **private network** (no public access in MVP)
- Should be behind authenticated API Gateway (not yet implemented)

### Service Account Permissions

The Cloud Run service account requires:

- Network access to Cloud SQL (or external PostgreSQL)
- No special GCP API permissions required

### Scaling

- Auto-scales based on request load
- Cold start optimized with Rust's fast startup time
- Stateless design enables horizontal scaling

---

## Known Issues & Limitations

### 🔴 Critical Issues

- **No authorization** – All endpoints publicly accessible
- **No ownership validation** – Anyone can modify any group
- **Creator not auto-added** – Group creator must manually join as member
- **Role not settable** – Can only add members with `"member"` role

### 🟡 Operational Gaps

- **Hardcoded port** – `PORT` env variable is ignored
- **Missing CI/CD** – GitHub Actions workflow not present in main branch
- **Missing Gitleaks** – Security scanning config not present
- **Metadata not exposed** – `assigned_at`, `due_date` in DB but not in API

### 🟡 Business Logic Limitations

- **No role management** – Cannot promote member to admin/teacher
- **No cascade deletes** – Deleting group doesn't clean up members/assignments
- **No membership queries** – Cannot list groups for a user
- **No visibility control** – No public/private group distinction

---

## Version Differences (develop vs main)

### Port Change

- `develop`: Port `8006`
- `main`: Port `3006` ✅ (aligned with infra)

### SQLx Version

- `develop`: SQLx `0.7`
- `main`: SQLx `0.8` ✅ (updated)

### Missing from main

- `.github/workflows/ci.yml` ❌
- `.gitleaks.toml` ❌

**Recommendation:** Restore CI/CD and Gitleaks from develop branch.

---

## TODO / Roadmap

### High Priority (MVP → Production)

- [ ]  **Add Gateway authentication** (extract `user_id`, `creator_id` from headers)
- [ ]  **Fix hardcoded port** (respect `PORT` env variable)
- [ ]  **Auto-add creator** (automatically add creator as owner on group creation)
- [ ]  **Add role management** (endpoint to change member roles)
- [ ]  **Restore CI/CD** (bring back GitHub Actions workflow)

### Medium Priority (Production Hardening)

- [ ]  **Add authorization checks** (only owner/admin/teacher can modify group)
- [ ]  **Expose assignment metadata** (`assigned_at`, `due_date` in API)
- [ ]  **Add membership queries** (list groups for a user)
- [ ]  **Add visibility control** (public vs private groups)

### Low Priority (Future Enhancements)

- [ ]  **Add cascade deletes** (clean up members/assignments on group delete)
- [ ]  **Add group invitations** (invite-only groups)
- [ ]  **Add activity tracking** (member activity logs)
- [ ]  **Add group statistics** (member count, completion rates)

---

## Project Status

**⚠️ Current Status: MVP (No Authorization)**

This microservice is **functional for MVP deployment** with core group management operational. Critical authorization gaps must be addressed before production.

**Known limitations to address for production:**

1. Add Gateway-based authorization
2. Fix hardcoded port binding
3. Auto-add creator as owner on group creation
4. Add role management endpoints
5. Restore CI/CD and security scanning
6. Expose assignment metadata in API

**Maintainers:** Altaïr Platform Team

---

## Notes

- **Port 3006** – Default port, currently hardcoded (ignores env variable)
- **No auth** – MVP accepts IDs from request body (security risk)
- **Idempotent adds** – Adding members/labs twice succeeds silently
- **Role limitations** – Can only add members with `"member"` role
- **Creator not member** – Must manually add creator to group after creation

---

## License

Internal Altaïr Platform Service – Not licensed for external use.
