# ScyllaDB TypeScript Driver

High-performance ScyllaDB and Apache Cassandra CQL driver for Node.js, built on the official [ScyllaDB Rust driver](https://rust-driver.docs.scylladb.com/stable/) via NAPI.

Requires **Node.js 18+**.

## Installation

```bash
npm install scylladb-driver
```

npm installs the correct native binary for your platform automatically:

| Platform | npm package |
|----------|-------------|
| Linux x64 (glibc) | `scylladb-driver-linux-x64-gnu` |
| macOS x64 | `scylladb-driver-darwin-x64` |
| Windows x64 | `scylladb-driver-win32-x64-msvc` |

## Quick start

```typescript
import { Cluster } from "scylladb-driver";

const cluster = new Cluster({
  nodes: ["127.0.0.1:9042"],
  localDatacenter: "datacenter1",
});

const session = await cluster.connect();

const result = await session.execute("SELECT release FROM system.local");
console.log(result.rows[0]);
```

## TypeScript helpers

Optional typed helpers are available from the secondary entry point:

```typescript
import { Cluster, getFirstRow } from "scylladb-driver/ts";

const session = await new Cluster({ nodes: ["127.0.0.1:9042"] }).connect();
const row = getFirstRow(await session.execute("SELECT release FROM system.local"));
```

## Examples

### Prepared statements

```typescript
const prepared = await session.prepare(
  "INSERT INTO users (id, name) VALUES (?, ?)"
);
await prepared.execute([1, "Alice"]);
```

### Paging

```typescript
let token: Buffer | null = null;
do {
  const page = await session.querySinglePage(
    "SELECT * FROM large_table",
    null,
    100,
    token
  );
  console.log(page.rows);
  token = page.nextPageToken ?? null;
} while (token);
```

### Batch statements

```typescript
const batch = session.batch("logged");
batch.add({ query: "INSERT INTO t (id, val) VALUES (?, ?)", params: [1, "a"] });
batch.add({ query: "INSERT INTO t (id, val) VALUES (?, ?)", params: [2, "b"] });
await batch.execute();
```

### Configuration

```typescript
const cluster = new Cluster({
  nodes: ["10.0.0.1:9042", "10.0.0.2:9042"],
  username: "admin",
  password: "secret",
  compression: "lz4",
  defaultKeyspace: "my_app",
  localDatacenter: "us-east-1",
  connectionTimeoutMs: 5000,
  executionProfile: {
    consistency: "local_quorum",
    requestTimeoutMs: 10000,
    retryPolicy: "default",
  },
});
```

## Features

- Async connect and query API
- Shard-aware routing (via the Rust driver)
- Prepared, batch, and paged queries
- Lightweight transactions (`wasApplied` on results)
- Execution profiles, retry policies, speculative execution
- TLS, compression (LZ4, Snappy), authentication
- Schema metadata, metrics, query tracing, execution history
- CQL scalar and collection types (lists, maps, sets, tuples, UDTs, blobs)

For advanced topics (load balancing, retry policies, data types, tracing), see the [Rust driver documentation](https://rust-driver.docs.scylladb.com/stable/). This package mirrors that API surface for Node.js.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## References

- [ScyllaDB Rust Driver docs](https://rust-driver.docs.scylladb.com/stable/)
- [ScyllaDB CQL reference](https://docs.scylladb.com/stable/cql/)
- [ScyllaDB drivers overview](https://www.scylladb.com/product/scylla-drivers/)

## License

MIT
