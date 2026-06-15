# ScyllaDB TypeScript Driver

A high-performance ScyllaDB/Cassandra CQL driver for Node.js, built on the official [Rust driver](https://rust-driver.docs.scylladb.com/stable/) via NAPI for native shard-aware performance.

Supports Node.js 18+.

## Installation

Stable (once released):

```bash
npm install scylladb-driver
```

Preview (`1.0.0-preview.x`):

```bash
npm install scylladb-driver@preview
```

## Quick Start

```typescript
import { Cluster } from "scylladb-driver";

const cluster = new Cluster({
  nodes: ["127.0.0.1:9042"],
  localDatacenter: "datacenter1",
});

const session = await cluster.connect();

// Simple query
const result = await session.execute("SELECT * FROM system.local");

// Parameterized query
await session.execute(
  "INSERT INTO my_table (id, name) VALUES (?, ?)",
  [1, "hello"]
);

// Prepared statement (server-side caching)
const prepared = await session.prepare(
  "INSERT INTO my_table (id, name) VALUES (?, ?)"
);
await prepared.execute([2, "world"]);
```

## Features

| Feature | Status | Notes |
|---------|--------|-------|
| Async connect and query | Done | Full async/await API |
| Shard-aware routing | Done | Automatic via Rust driver |
| Prepared statements | Done | `session.prepare()` / `prepared.execute()` |
| Batch statements | Done | Logged, unlogged, counter |
| Paging | Done | `session.querySinglePage()` with token |
| Lightweight transactions (LWT) | Done | `wasApplied` on results |
| Keyspace switching | Done | `session.useKeyspace()` |
| Schema agreement | Done | `session.awaitSchemaAgreement()` |
| Execution profiles | Done | Consistency, timeout, retry, speculative exec |
| Retry policies | Done | Default, downgrading consistency, fallthrough |
| Speculative execution | Done | Simple speculative policy |
| Load balancing | Done | Datacenter-aware via Rust driver default policy |
| All CQL data types | Done | Scalars, lists, sets, maps, tuples, blobs, UDTs |
| Schema metadata | Done | `getKeyspaces()`, `getTable()`, `refreshMetadata()` |
| Metrics | Done | `session.getMetrics()` |
| Query tracing | Done | `tracingId` on results, `getTracingInfo()` |
| Compression | Done | LZ4, Snappy |
| Authentication | Done | Username/password |
| TLS | Done | `rustls` via `tls.caFilepath` on `ClusterConfig` |
| Connection pool tuning | Done | Timeout, TCP keepalive, nodelay |
| Query execution history | Done | `session.executeWithHistory()` |
| Async paging (all pages) | Done | `session.executePaged()` |
| CDC consumer | Out of scope | Separate track; CQL driver only |
| Sync API | Not planned | Async-only (matches Rust/Python drivers) |
| Alternator (DynamoDB API) | Out of scope | Separate package recommended |

## Configuration

```typescript
import { Cluster } from "scylladb-driver";

const cluster = new Cluster({
  nodes: ["10.0.0.1:9042", "10.0.0.2:9042"],
  username: "admin",
  password: "secret",
  compression: "lz4",
  defaultKeyspace: "my_app",
  localDatacenter: "us-east-1",
  connectionTimeoutMs: 5000,
  tcpNodelay: true,
  tcpKeepaliveIntervalMs: 30000,
  disallowShardAwarePort: false,
  schemaAgreementTimeoutMs: 30000,
  executionProfile: {
    consistency: "local_quorum",
    requestTimeoutMs: 10000,
    retryPolicy: "default",
    speculativeExecution: {
      maxRetryCount: 2,
      retryIntervalMs: 100,
    },
  },
});
```

## Paging

```typescript
let pagingState = null;
do {
  const page = await session.querySinglePage(
    "SELECT * FROM large_table",
    null,
    100, // page size
    pagingState
  );
  console.log(page.rows);
  pagingState = page.nextPageToken;
} while (pagingState);
```

## Batch Statements

```typescript
const batch = session.batch("logged"); // "logged" | "unlogged" | "counter"
batch.add({ query: "INSERT INTO t (id, val) VALUES (?, ?)", params: [1, "a"] });
batch.add({ query: "INSERT INTO t (id, val) VALUES (?, ?)", params: [2, "b"] });
await batch.execute();
```

## Schema Introspection

```typescript
const keyspaces = session.getKeyspaces();
const table = session.getTable("my_keyspace", "my_table");
console.log(table.partitionKey, table.clusteringKey, table.columns);
```

## Metrics and Tracing

```typescript
const metrics = session.getMetrics();
console.log(`Queries: ${metrics.queriesNum}, Errors: ${metrics.errorsNum}`);

const result = await session.execute("SELECT ...", null, { tracing: true });
const info = await session.getTracingInfo(result.tracingId);
console.log(info.coordinator, info.durationUs, info.events);
```

## Development

```bash
# Start ScyllaDB
yarn docker:up

# Build native bindings
yarn build:debug

# Run unit tests
yarn test

# Run integration tests (requires ScyllaDB running)
yarn test:integration

# Run all tests
yarn test:all

# Stop ScyllaDB
yarn docker:down
```

## Publishing

See [PUBLISHING.md](PUBLISHING.md) for the full release guide, including the **1.0.0-preview.1** launch checklist.

Quick reference:

```bash
yarn pack:local          # build + pack + verify tarball locally
yarn pack:ci             # pack + verify (CI, after artifacts)
```

CI runs `pack-check` on every build and publishes to npm when the latest commit message is a semver version (preview releases use the `preview` dist-tag).

## Architecture

This driver wraps the official [scylla-rust-driver](https://github.com/scylladb/scylla-rust-driver) (v1.7) through Node.js NAPI bindings. The Rust layer handles CQL protocol, connection pooling, shard-aware routing, and serialization. The TypeScript layer provides ergonomic APIs and type safety.

## References

- [ScyllaDB Drivers Product Page](https://www.scylladb.com/product/scylla-drivers/)
- [Rust Driver Documentation](https://rust-driver.docs.scylladb.com/stable/)
- [ScyllaDB CQL Reference](https://docs.scylladb.com/stable/cql/)

## License

MIT
