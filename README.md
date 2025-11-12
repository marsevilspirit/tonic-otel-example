# `tonic-otel-example`

A minimal example project demonstrating OpenTelemetry integration (distributed tracing and metrics) with a `tonic` gRPC client and server.

## Overview

This repository contains a simple `helloworld` gRPC service built with `tonic`. The client and server are instrumented with OpenTelemetry to export traces and metrics. A Docker Compose environment is provided to run a complete observability stack, including the OTel Collector, Jaeger, and Prometheus.

## Features

  * Tonic-based gRPC client (`helloworld-client`) and server (`helloworld-server`).
  * OpenTelemetry tracing configured for both client and server.
  * Trace context propagation from client to server via gRPC metadata.
  * OpenTelemetry metrics (a request counter) implemented on the server.
  * A `docker-compose` setup including an OpenTelemetry Collector, Jaeger (for traces), and Prometheus (for metrics).

## Project Structure

```
.
├── Cargo.toml          # Rust dependencies
├── Cargo.lock          # Locked dependencies
├── build.rs            # Proto build script
├── docker/
│   ├── docker-compose.yml  # Observability stack (Collector, Jaeger, Prometheus)
│   ├── otel-config.yaml    # OpenTelemetry Collector configuration
│   └── prometheus.yml      # Prometheus scrape configuration
├── proto/
│   └── helloworld.proto    # gRPC service definition
└── src/
    ├── client.rs         # Client implementation
    └── server.rs         # Server implementation
```

## Usage

### 1\. Start the Observability Stack

The stack includes the OTel Collector, Jaeger, and Prometheus.

```sh
cd docker
docker-compose up -d
```

### 2\. Run the gRPC Server

Open a new terminal to run the server.

```sh
cargo run --bin helloworld-server
```

### 3\. Run the gRPC Client

Open a third terminal to run the client and make a request.

```sh
cargo run --bin helloworld-client
```

### 4\. View Results

After running the client, you can inspect the telemetry data:

  * **Traces (Jaeger):**
    Navigate to `http://localhost:16686` and find traces for the `helloworld-client` or `helloworld-server` service.

  * **Metrics (Prometheus):**
    Navigate to `http://localhost:9090`. You can execute a query for the server-side counter, e.g., `greeter_requests_total`.

  * **ZPages (OTel Collector):**
    Navigate to `http://localhost:55679` to view internal collector diagnostics and debugging information.
