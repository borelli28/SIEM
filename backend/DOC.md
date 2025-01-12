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
  * Processes incoming log entries
  * Manages log batch processing
  * Validates CEF (Common Event Format) logs
  * Triggers rule evaluation

- **Message Queue** (`message_queue.rs`)
  * Implements asynchronous log processing
  * Manages batch queuing and dequeuing
  * Ensures ordered log processing

### 4. Security Components
- **Authentication** (`account.rs`, `auth_session.rs`)
  * User account management
  * Session handling
  * Session timeout at 20 minutes if no activity
  * Password hashing with Argon2
  * Role-based access control

- **CSRF Protection** (`csrf.rs`)
  * Token generation and validation
  * Form protection
  * Session-based security

### 5. Rule Engine (`rules.rs`)
- Manages detection rules
- Implements SIGMA-like rule format
- Handles rule evaluation against logs
- Manages rule lifecycle (CRUD operations)

### 6. Agent Management (`agent.rs`)
- Agent registration and authentication
- API key management
- Heartbeat monitoring

## Basic Workflow

1. **Authentication Flow**:
   - Client authenticates via login endpoint
   - Session is established with security tokens
   - CSRF protection is activated for subsequent requests

2. **Log Ingestion**:
   - Logs are received via agent or direct upload
   - Logs are batched (50 entries per batch)
   - Batches are queued for processing

3. **Log Processing**:
   - Queued batches are processed asynchronously
   - Each log entry is validated and normalized
   - Logs are stored in the database
   - Sigma rules are evaluated against new logs

4. **Alert Generation**:
   - Matching rules trigger alert creation

## API Structure

The backend provides RESTful endpoints for:
- Account management
- Host management
- Rule management
- Log querying and filtering
- Alert handling
- Agent operations
- Session management