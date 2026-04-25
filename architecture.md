# Photo App RS - Architecture

A Rust-based photo and video management system for organizing media files from SD cards and storage devices.

---

## Table of Contents

- [Overview](#overview)
- [Project Structure](#project-structure)
- [Data Flow](#data-flow)
- [Module Breakdown](#module-breakdown)
- [Database Schema](#database-schema)
- [Dependencies](#dependencies)
- [Design Patterns](#design-patterns)

---

## Overview

| Aspect | Description |
|--------|-------------|
| **Purpose** | Organize media files into structured directories, detect duplicates, extract metadata |
| **Language** | Rust |
| **UI Framework** | Iced (MVU pattern) |
| **Database** | SQLite (rusqlite) |
| **Architecture** | Synchronous I/O with signal-based graceful shutdown |

---

## Project Structure

```
photo_app_rs/
├── Cargo.toml                    # Workspace root
├── src/
│   └── main.rs                   # Iced GUI (placeholder)
│
├── analytics/                    # Core application
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Entry point & orchestration
│       ├── lib.rs
│       ├── database/
│       │   ├── mod.rs
│       │   ├── connection.rs     # DB init & connection
│       │   ├── migrations.rs     # Schema versioning
│       │   ├── models.rs         # Data structures
│       │   ├── operations.rs     # CRUD operations
│       │   └── migrations/
│       │       └── 001_initial_schema.sql
│       └── utils/
│           ├── mod.rs
│           ├── core.rs           # File scanning & metadata
│           └── duplicates.rs     # Hash-based deduplication
│
└── misc/                         # Utility binaries
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── bin/
            └── extract_meta.rs   # Video metadata testing
```

---

## Data Flow

```mermaid
flowchart TD
    subgraph Input
        A[SD Card / Storage] --> B[File System]
    end

    subgraph Scanning
        B --> C[scan_directory]
        C --> D{File Type?}
        D -->|Image| E[Extract EXIF]
        D -->|Video| F[Extract Video Meta]
        D -->|Other| G[Skip]
        E --> H[Create Media Object]
        F --> H
    end

    subgraph Processing
        H --> I[Calculate SHA256 Hash]
        I --> J[Insert to media_files]
        J --> K[find_duplicates]
        K --> L[Group by Hash]
        L --> M[Insert duplicate_groups]
    end

    subgraph Output
        M --> N[Export CSV]
        N --> O[Copy to Destination]
        O --> P[year/month/day/type/]
    end

    subgraph Database
        Q[(SQLite)]
        J --> Q
        M --> Q
        R[backup_sessions] --> Q
    end

    subgraph Lifecycle
        S[Start] --> T[Create Backup Session]
        T --> C
        O --> U{Completed?}
        U -->|Yes| V[Mark Completed]
        U -->|Ctrl+C| W[Mark Cancelled]
        V --> X[End]
        W --> X
    end
```

---

## Module Breakdown

### 1. Analytics Package (Core)

#### `main.rs` - Orchestration
```
Responsibilities:
├── Initialize database connection
├── Create backup session record
├── Set up Ctrl+C signal handler
├── Coordinate scanning → processing → export
├── Calculate wasted space statistics
└── Update session status on completion
```

#### `database/` - Persistence Layer

| File | Purpose |
|------|---------|
| `connection.rs` | Initialize SQLite, enable foreign keys, run migrations |
| `migrations.rs` | Version-controlled schema changes with tracking table |
| `models.rs` | `MediaFileRow`, `DuplicateGroupRow`, `BackupSessionRow` |
| `operations.rs` | Insert media, manage sessions, record duplicates |

#### `utils/` - Core Logic

| File | Key Functions |
|------|---------------|
| `core.rs` | `scan_directory()`, `Media::new()`, `FileType::from_path()` |
| `duplicates.rs` | `find_duplicates()`, `calculate_hash()`, `final_path_for_media()` |

### 2. GUI Package (Root)

```rust
// Iced MVU Architecture
struct Counter { value: i32 }
enum Message { Increment, Decrement }
fn update(counter: &mut Counter, message: Message)
fn view(counter: &Counter) -> Element<Message>
```

### 3. Misc Package

- `extract_meta.rs`: Standalone binary for testing video metadata extraction

---

## Database Schema

```mermaid
erDiagram
    media_files {
        int id PK
        string hash UK
        int file_size_bytes
        string media_type
        string extension
        string camera_make
        string camera_model
        string lens_model
        datetime date_taken
        int iso
        string aperture
        string shutter_speed
        string focal_length
        string software
        float duration_seconds
        int resolution_width
        int resolution_height
        string path
        datetime date_added
        datetime date_modified
        datetime created_at
        datetime updated_at
    }

    duplicate_groups {
        int id PK
        int media_file_id FK
        int total_copies
        int total_size_bytes
        int wasted_space_bytes
        datetime created_at
    }

    backup_sessions {
        int id PK
        string source_path
        string destination_path
        int files_scanned
        int files_copied
        int files_skipped
        int bytes_copied
        datetime started_at
        datetime completed_at
        int duration_seconds
        string status
        string error_message
        datetime created_at
    }

    __schema_migrations {
        string version PK
        datetime applied_at
    }

    media_files ||--o{ duplicate_groups : "has duplicates"
```

### Indices

| Table | Index | Columns |
|-------|-------|---------|
| media_files | idx_media_files_hash | hash |
| media_files | idx_media_files_date_taken | date_taken |
| media_files | idx_media_files_media_type | media_type |
| media_files | idx_media_files_path | path |
| duplicate_groups | idx_duplicate_groups_media_file_id | media_file_id |
| backup_sessions | idx_backup_sessions_status | status |
| backup_sessions | idx_backup_sessions_started_at | started_at |

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `iced` | 0.14.0 | Cross-platform GUI framework |
| `rusqlite` | 0.38.0 | SQLite database driver |
| `kamadak-exif` | 0.6.1 | EXIF metadata extraction |
| `media_info` | 0.6.0 | Video metadata (duration, resolution) |
| `sha2` | 0.10.9 | SHA256 hashing for deduplication |
| `chrono` | 0.4.42 | Date/time handling |
| `csv` | 1.3 | CSV export |
| `serde` | 1.0 | Serialization |
| `ctrlc` | 3.4 | Graceful shutdown signals |

---

## Design Patterns

### Repository Pattern
```
database/operations.rs encapsulates all DB operations
├── new_backup_session()
├── insert_media_file()
├── insert_duplicate_group()
├── mark_session_completed()
└── mark_session_cancelled()
```

### Factory Pattern
```
FileType::from_path(path) → FileType
Media::new(path) → Media
```

### Migration Pattern
```
Version-controlled schema with __schema_migrations
├── Check applied versions
├── Apply pending migrations in order
└── Track each application timestamp
```

### Signal Handling
```rust
Arc<Mutex<Option<i64>>>  // Thread-safe session ID
ctrlc::set_handler()     // Capture Ctrl+C
mark_session_cancelled() // Clean shutdown
```

---

## Processing Pipeline Detail

```mermaid
flowchart LR
    subgraph Phase1[" 1. Scan "]
        A1[Walk Directory] --> A2[Filter Extensions]
        A2 --> A3[Collect Paths]
    end

    subgraph Phase2[" 2. Extract "]
        B1[Read File] --> B2{Type}
        B2 -->|jpg/png/heic| B3[EXIF Reader]
        B2 -->|mp4/mov| B4[MediaInfo]
        B3 --> B5[Media Struct]
        B4 --> B5
    end

    subgraph Phase3[" 3. Hash "]
        C1[Open File] --> C2[Read 128KB]
        C2 --> C3[SHA256]
        C3 --> C4[Hex String]
    end

    subgraph Phase4[" 4. Dedupe "]
        D1[Group by Hash] --> D2[Count Copies]
        D2 --> D3[Calculate Waste]
        D3 --> D4[Record Groups]
    end

    subgraph Phase5[" 5. Organize "]
        E1[Parse Date] --> E2[Build Path]
        E2 --> E3[year/month/day/type/]
        E3 --> E4[Copy File]
    end

    Phase1 --> Phase2 --> Phase3 --> Phase4 --> Phase5
```

---

## File Organization Output

```
destination/
├── 2024/
│   ├── 01/
│   │   ├── 15/
│   │   │   ├── images/
│   │   │   │   ├── IMG_001.jpg
│   │   │   │   └── IMG_002.heic
│   │   │   └── videos/
│   │   │       └── VID_001.mp4
│   │   └── 16/
│   │       └── images/
│   │           └── IMG_003.png
│   └── 02/
│       └── ...
└── 2023/
    └── ...
```

---

## Concurrency Model

```
┌─────────────────────────────────────────┐
│              Main Thread                │
├─────────────────────────────────────────┤
│  ┌─────────────────────────────────┐    │
│  │     Arc<Mutex<session_id>>      │◄───┼── Shared state
│  └─────────────────────────────────┘    │
│                  │                      │
│    ┌─────────────┴─────────────┐        │
│    ▼                           ▼        │
│  Normal                    Ctrl+C       │
│  Execution                 Handler      │
│    │                           │        │
│    ▼                           ▼        │
│  mark_completed()      mark_cancelled() │
└─────────────────────────────────────────┘

Note: Single-threaded synchronous I/O
      No async runtime or thread pool
```

---

## Configuration

| Setting | Current Value | Location |
|---------|---------------|----------|
| Source Path | `/Users/saujanya/sandisk_media` | `analytics/src/main.rs` |
| Database | `./.photo_app_rs/sqlite.db` | `database/connection.rs` |
| CSV Export | `./csv_exports/final_export.csv` | `analytics/src/main.rs` |
| Destination | `{source}/final_export` | `analytics/src/main.rs` |
| Hash Size | 128 KB (partial file) | `utils/duplicates.rs` |

---

## Supported Formats

### Images
`jpg`, `jpeg`, `png`, `gif`, `bmp`, `tiff`, `webp`, `heic`, `heif`, `raw`, `cr2`, `nef`, `arw`, `dng`

### Videos
`mp4`, `mov`, `avi`, `mkv`, `wmv`, `flv`, `webm`, `m4v`, `3gp`

---

## Future Considerations

- [ ] Async I/O with tokio for parallel processing
- [ ] Rayon for parallel file hashing
- [ ] Configuration file support (TOML/YAML)
- [ ] Full GUI implementation
- [ ] Cloud backup integration
- [ ] Face detection / ML-based organization
