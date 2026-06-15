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

test("useKeyspace switches keyspace", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    await session.execute(
      "CREATE KEYSPACE IF NOT EXISTS phase1_ks_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
    );
    await session.useKeyspace("phase1_ks_test");
    await session.execute(
      "CREATE TABLE IF NOT EXISTS ks_check (id int PRIMARY KEY, val text)"
    );
    await session.execute("INSERT INTO ks_check (id, val) VALUES (1, 'hello')");
    const result = await session.execute(
      "SELECT val FROM ks_check WHERE id = 1"
    );
    t.is(result.rows[0].val, "hello");
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("awaitSchemaAgreement returns a UUID", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const uuid = await session.awaitSchemaAgreement();
    t.regex(uuid, /^[0-9a-f]{8}-/);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("checkSchemaAgreement returns a UUID or null", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const uuid = await session.checkSchemaAgreement();
    if (uuid !== null) {
      t.regex(uuid, /^[0-9a-f]{8}-/);
    } else {
      t.pass("Schema not yet in agreement (null)");
    }
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("execute with consistency option", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local",
      null,
      { consistency: "one" }
    );
    t.true(result.rowLength >= 1);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("ClusterConfig with connectionTimeout", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    connectionTimeoutMs: 10000,
    tcpNodelay: true,
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

test("connect with default keyspace in config", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    defaultKeyspace: "system",
  });
  try {
    const session = await cluster.connect();
    const result = await session.execute("SELECT key FROM local LIMIT 1");
    t.true(result.rowLength >= 1);
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});
