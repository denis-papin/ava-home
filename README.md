# AVA Home

AVA Home is a personal home-automation platform for observing, storing, and controlling the devices that run a connected home.

It is built around a simple idea: every device event should be captured, normalized, persisted, and made available to automation services and dashboards. Zigbee sensors, MQTT topics, radiator commands, PostgreSQL history, and a web dashboard all work together as one local system.

## What It Does

- Listens to Zigbee2MQTT and other MQTT device topics.
- Stores home events and device states in PostgreSQL.
- Computes heating regulation decisions from room temperatures and target modes.
- Controls Heatzy radiators through dedicated Rust services and APIs.
- Exposes dashboard data through an Axum HTTP API.
- Provides a React + ReScript dashboard for real-time room status and heating analysis.
- Shares common automation primitives through reusable Rust toolkit crates.

## Architecture

```text
Zigbee / Home devices
        |
        v
   MQTT broker
        |
        +-------------------+
        |                   |
        v                   v
 event-storage        regulator services
        |                   |
        v                   v
   PostgreSQL       radiator-api / Heatzy
        |                   ^
        v                   |
   dashboard-api -----------+
        |
        v
   re-dashboard
```

The system is intentionally modular. Long-running services subscribe to MQTT topics, transform messages into domain events, and persist the latest known state. API services then expose that state to the dashboard or apply explicit commands, such as changing a radiator mode.

## Workspace Layout

| Path | Purpose |
| --- | --- |
| `ava-toolkit` | Shared MQTT/device loop abstractions used by automation services. |
| `common-config` | Configuration loading helpers for service property files. |
| `commons-error` | Shared error helpers and logging macros. |
| `commons-pg` | PostgreSQL transaction and connection utilities. |
| `event-storage` | MQTT listener that records device events into PostgreSQL. |
| `regulator` | Heating regulation logic based on room temperature and target state. |
| `regulator-heart-beat` | Publishes or coordinates periodic regulation updates. |
| `radiator-ctrl` | MQTT-oriented radiator control service. |
| `radiator-api` | HTTP API for direct radiator commands and computed radiator updates. |
| `radiator-toolkit` | Heatzy client logic shared by radiator services. |
| `dashboard-api` | Axum API serving room state, heating plans, and dashboard actions. |
| `re-dashboard` | React + ReScript frontend consuming `dashboard-api`. |
| `mqtt-bridge` | Bridge between MQTT traffic and websocket clients. |
| `luminator` | Automation service for lighting-oriented device loops. |
| `mqtt5-r` | MQTT v5 experimentation/service code. |

## Main Services

### `event-storage`

Subscribes to configured MQTT devices and stores incoming events. This gives the rest of the platform a reliable event history and a source of latest known states.

### `regulator`

Reads temperature information, computes radiator regulation state, and publishes decisions for the heating system.

### `radiator-api`

Provides HTTP endpoints for radiator control:

- `POST /radiator-api/radiator/:room`
- `POST /radiator-api/update-radiator`

It talks to Heatzy through `radiator-toolkit`, avoids unnecessary commands when the persisted state is already correct, and records state changes after successful updates.

### `dashboard-api`

Provides HTTP endpoints consumed by the frontend:

- `GET /dashboard-api/index/data`
- `POST /dashboard-api/index/radiator/:room`
- `GET /dashboard-api/heating_plan`
- `GET /dashboard-api/heating_plan_by_room`
- `GET /dashboard-api/room_temperature_by_mode`

It aggregates PostgreSQL data, current regulation state, and radiator API actions behind a dashboard-friendly interface.

### `re-dashboard`

A standalone React + ReScript frontend showing the home state for the main rooms:

- living room (`salon`)
- office (`bureau`)
- bedroom (`chambre`)
- hallway (`couloir`)

It displays temperatures, radiator state, refreshes automatically, and includes heating analysis screens.

## Technology

- Rust workspace with multiple services and shared crates.
- Tokio for async runtimes.
- Axum for HTTP APIs.
- Rumqttc for MQTT v5 communication.
- PostgreSQL for event and state storage.
- SQLx and postgres clients for database access.
- Reqwest for Heatzy and internal HTTP calls.
- React, ReScript, and Vite for the dashboard.

## Build

Build the Rust workspace:

```bash
cargo build
```

Run tests where available:

```bash
cargo test
```

Build the dashboard:

```bash
cd re-dashboard
npm install
npm run build
```

## Run Locally

Each Rust service reads its own configuration from property files selected through environment variables. The exact configuration depends on the target environment, but the usual pattern is:

```bash
cargo run -p dashboard-api
cargo run -p radiator-api
cargo run -p event-storage
cargo run -p regulator
```

Run the frontend during development:

```bash
cd re-dashboard
npm install
npm run dev
```

By default, the dashboard development server runs on `http://localhost:2091` and calls `dashboard-api` at `http://127.0.0.1:2090/dashboard-api`. Override the API origin with:

```bash
VITE_DASHBOARD_API_ORIGIN=http://127.0.0.1:2090 npm run dev
```

## Design Principles

- Device logic is isolated in small services.
- MQTT is the nervous system of the home.
- PostgreSQL is the memory of the home.
- APIs expose stable, dashboard-friendly views over event data.
- Shared crates keep configuration, database, MQTT, and radiator code consistent.
- Commands should be idempotent where possible: do not call a physical device when the desired state is already known.

## Project Status

AVA Home is an active personal automation workspace. Some crates are production services, some are support libraries, and some are experimental or transitional. The current direction is toward clearer service boundaries:

- `dashboard-api` as the main read/control API for the UI.
- `radiator-api` as the dedicated radiator command API.
- `ava-toolkit`, `common-config`, and `radiator-toolkit` as reusable building blocks.
- `re-dashboard` as the modern dashboard frontend.
