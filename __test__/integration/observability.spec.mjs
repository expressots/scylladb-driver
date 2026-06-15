import test from "ava";
import { Cluster } from "../../index.js";

const SCYLLA_NODE = process.env.SCYLLA_NODE || "127.0.0.1:9042";

function skipIfNoScylla(t, error) {
  if (
    error instanceof Error &&
    error.message.includes("Failed to connect to cluster")
  ) {
    t.pass(`Skipped: Scylla not available at ${SCYLLA_NODE}`);
    return true;
  }
  return false;
}

test("getMetrics returns driver metrics", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    // Execute a query to generate some metrics
    await session.execute("SELECT release_version FROM system.local");
    await session.execute("SELECT release_version FROM system.local");

    const metrics = session.getMetrics();
    t.is(typeof metrics.queriesNum, "number");
    t.true(metrics.queriesNum >= 2);
    t.is(typeof metrics.errorsNum, "number");
    t.is(typeof metrics.totalConnections, "number");
    t.true(metrics.totalConnections >= 1);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("tracing returns tracingId when enabled", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local",
      null,
      { tracing: true }
    );
    t.truthy(result.tracingId);
    t.regex(result.tracingId, /^[0-9a-f]{8}-/);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("getTracingInfo returns trace details", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local",
      null,
      { tracing: true }
    );

    // Wait a bit for tracing data to propagate
    await new Promise((r) => setTimeout(r, 100));

    const info = await session.getTracingInfo(result.tracingId);
    t.truthy(info);
    t.is(typeof info.coordinator, "string");
    t.is(typeof info.durationUs, "number");
    t.true(Array.isArray(info.events));
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("tracingId is null when tracing is not enabled", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local"
    );
    t.is(result.tracingId ?? null, null);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("disallowShardAwarePort config works", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    disallowShardAwarePort: true,
  });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local"
    );
    t.true(result.rowLength >= 1);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("executeWithHistory returns execution history", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const { result, history } = await session.executeWithHistory(
      "SELECT release_version FROM system.local"
    );
    t.true(result.rowLength >= 1);
    t.true(Array.isArray(history));
    t.true(history.length >= 1);
    t.true(history[0].attempts.length >= 1);
    t.truthy(history[0].attempts[0].nodeAddress);
    t.is(history[0].attempts[0].success, true);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("executePaged fetches all rows across pages", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    await session.execute(
      "CREATE KEYSPACE IF NOT EXISTS paged_all_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
    );
    await session.useKeyspace("paged_all_test");
    await session.execute(
      "CREATE TABLE IF NOT EXISTS items (id int PRIMARY KEY, val text)"
    );
    for (let i = 0; i < 20; i++) {
      await session.execute("INSERT INTO items (id, val) VALUES (?, ?)", [
        i,
        `v${i}`,
      ]);
    }
    const result = await session.executePaged(
      "SELECT * FROM items",
      null,
      5
    );
    t.is(result.rowLength, 20);
    t.is(result.rows.length, 20);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("timestamp generator monotonic works", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    timestampGenerator: "monotonic",
  });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local"
    );
    t.true(result.rowLength >= 1);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("per-statement execution profile override", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local",
      null,
      {
        executionProfile: {
          consistency: "one",
          requestTimeoutMs: 30000,
        },
      }
    );
    t.true(result.rowLength >= 1);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});
