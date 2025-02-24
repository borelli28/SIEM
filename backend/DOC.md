# SIEM Backend Documentation

## Components

### 1. Core Services (`main.rs`)
- Entry point for the application
- Configures HTTP server and middleware
- Sets up CORS, session management, and CSRF protection
- Manages API routing and endpoint handlers

### 2. Database Management (`database.rs`, `schema.rs`)
- SQLite database connection handling
- Schema creation and management for:
  * Accounts
  * Rules
  * Hosts
  * Alerts
  * Logs
  * Agents

### 3. Log Collection System
- **Batch Maker** (`batch_maker.rs`)
  * Handles log file processing
  * Creates batches of up to 50 log entries
  * Manages queue integration

- **Collector** (`collector.rs`)
  * Dequeues log batches from the message queue
  * Processes each log by invoking `log_parser.rs` for parsing
  * Constructs `Log` structs and passes them to `log.rs` for storage
  * Post-storage, evaluates each log against detection rules using `rules.rs`
  * Manages deduplication and alert generation

- **Log Parser** (`log_parser.rs`)
  * Ingests individual log strings
  * Cleans logs (e.g., trims whitespace, collapses multi-line entries)
  * Detects log formats (CEF, Syslog, JSON) using heuristics
  * Parses logs into a `NormalizedLog` structure and returns a JSON string with timestamp
  * Supports flexible key-value pair extraction via an `extensions` map

- **Log Storage** (`log.rs`)
  * Defines the `Log` struct: `id`, `hash`, `account_id`, `host_id`, `timestamp`, `log_data` (JSON string)
  * Provides `create_log` to insert logs into the database, hashing `log_data` for deduplication
  * Supports querying logs by account ID or custom EQL queries

- **Message Queue** (`message_queue.rs`)
  * Implements asynchronous log processing
  * Manages batch queuing and dequeuing
  * Ensures ordered log processing

### 4. Security Components
- **Authentication** (`account.rs`, `auth_session.rs`)
  * User account management
  * Session handling with 20-minute inactivity timeout
  * Password hashing with Argon2
  * Role-based access control

- **CSRF Protection** (`csrf.rs`)
  * Token generation and validation
  * Form protection
  * Session-based security

### 5. Rule Engine (`rules.rs`)
- Manages Sigma-like detection rules
- Evaluates rules against `NormalizedLog` structs post-storage
- Matches rule conditions using top-level fields (`event_type`, `src_ip`, etc.) and `extensions`
- Generates alerts for matching logs
- Handles rule lifecycle (CRUD operations)

### 6. Agent Management (`agent.rs`)
- Agent registration and authentication
- API key management
- Heartbeat monitoring

## Basic Workflow

1. **Authentication Flow**:
   - Client authenticates via login endpoint
   - Session established with `auth_session` cookie

2. **Log Ingestion**:
   - Logs received via agent or direct upload
   - `batch_maker.rs` batches logs (up to 50 per batch)
   - Batches queued in `message_queue.rs`

3. **Log Processing**:
   - `collector.rs` dequeues a batch from the message queue
   - For each log in the batch:
     - `log_parser.rs` cleans the log, detects its format, and parses it into a `NormalizedLog` JSON string
     - `collector.rs` constructs a `Log` struct with the JSON and calls `log.rs::create_log`
     - `log.rs` validates, hashes the `log_data`, and inserts the log into the `logs` table
       - Duplicate logs (by hash) are skipped
     - `collector.rs` deserializes the stored `log_data` into `NormalizedLog` and calls `rules.rs::evaluate_log_against_rules`

4. **Alert Generation**:
   - `rules.rs` evaluates Sigma rules against each `NormalizedLog`
   - Matching rules trigger alert creation, stored in the database via `alert.rs`

## API Structure

The backend provides RESTful endpoints for:
- Account management
- Host management
- Rule management
- Log querying and filtering
- Alert handling
- Agent operations
- Session management