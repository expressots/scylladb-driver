import test from "ava";

import { Cluster } from "../../index.js";

const SCYLLA_NODE = process.env.SCYLLA_NODE || "127.0.0.1:9042";

test("connects and reads system.local", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });

  try {
    const session = await cluster.connect();
    const result = await session.execute(
      "SELECT release_version FROM system.local"
    );

    t.true(result.rowLength >= 1);
    t.true(Array.isArray(result.rows));
    t.true(result.columns.length >= 1);
  } catch (error) {
    if (
      error instanceof Error &&
      error.message.includes("Failed to connect to cluster")
    ) {
      t.pass(`Skipped: Scylla not available at ${SCYLLA_NODE}`);
      return;
    }

    throw error;
  }
});

test("creates keyspace, inserts rows, and reads them back", async (t) => {
  const cluster = new Cluster({ nodes: [SCYLLA_NODE] });

  try {
    const session = await cluster.connect();

    await session.execute(
      "CREATE KEYSPACE IF NOT EXISTS driver_integration_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
    );
    await session.execute("USE driver_integration_test");
    await session.execute(`
      CREATE TABLE IF NOT EXISTS integration_users (
        name text,
        age int,
        PRIMARY KEY (name)
      )
    `);

    await session.execute(
      "INSERT INTO integration_users (name, age) VALUES (?, ?)",
      ["integration-user", 42]
    );

    const result = await session.execute(
      "SELECT name, age FROM integration_users WHERE name = ?",
      ["integration-user"]
    );

    t.is(result.rowLength, 1);
    t.is(result.rows[0].name, "integration-user");
    t.is(result.rows[0].age, 42);
  } catch (error) {
    if (
      error instanceof Error &&
      error.message.includes("Failed to connect to cluster")
    ) {
      t.pass(`Skipped: Scylla not available at ${SCYLLA_NODE}`);
      return;
    }

    throw error;
  }
});
