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

test.before(async (t) => {
  try {
    const cluster = new Cluster({ nodes: [SCYLLA_NODE] });
    const session = await cluster.connect();
    await session.execute(
      "CREATE KEYSPACE IF NOT EXISTS types_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
    );
    await session.useKeyspace("types_test");
    await session.execute(`
      CREATE TABLE IF NOT EXISTS all_types (
        id int PRIMARY KEY,
        t_text text,
        t_int int,
        t_bigint bigint,
        t_float float,
        t_double double,
        t_boolean boolean,
        t_blob blob,
        t_list list<text>,
        t_set set<int>,
        t_map map<text, int>
      )
    `);
    t.context.session = session;
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("round-trip scalar types", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await session.execute(
    "INSERT INTO all_types (id, t_text, t_int, t_bigint, t_float, t_double, t_boolean) VALUES (?, ?, ?, ?, ?, ?, ?)",
    [1, "hello", 42, 2147483648, { __float: 3.14 }, 2.718281828, true]
  );

  const result = await session.execute(
    "SELECT * FROM all_types WHERE id = ?",
    [1]
  );
  const row = result.rows[0];
  t.is(row.t_text, "hello");
  t.is(row.t_int, 42);
  t.is(row.t_boolean, true);
  t.is(typeof row.t_double, "number");
});

test("bind and read blob as Buffer-style object", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const blobData = { type: "Buffer", data: [0x48, 0x65, 0x6c, 0x6c, 0x6f] };
  await session.execute(
    "INSERT INTO all_types (id, t_blob) VALUES (?, ?)",
    [2, blobData]
  );

  const result = await session.execute(
    "SELECT t_blob FROM all_types WHERE id = ?",
    [2]
  );
  const blob = result.rows[0].t_blob;
  t.is(blob.type, "Buffer");
  t.deepEqual(blob.data, [0x48, 0x65, 0x6c, 0x6c, 0x6f]);
});

test("bind and read list", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await session.execute(
    "INSERT INTO all_types (id, t_list) VALUES (?, ?)",
    [3, ["alpha", "beta", "gamma"]]
  );

  const result = await session.execute(
    "SELECT t_list FROM all_types WHERE id = ?",
    [3]
  );
  t.deepEqual(result.rows[0].t_list, ["alpha", "beta", "gamma"]);
});

test("bind and read map", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await session.execute(
    "INSERT INTO all_types (id, t_map) VALUES (?, ?)",
    [4, { x: 10, y: 20 }]
  );

  const result = await session.execute(
    "SELECT t_map FROM all_types WHERE id = ?",
    [4]
  );
  const map = result.rows[0].t_map;
  t.is(map.x, 10);
  t.is(map.y, 20);
});

test("getKeyspaces returns keyspace list", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const keyspaces = session.getKeyspaces();
  t.true(Array.isArray(keyspaces));
  t.true(keyspaces.length > 0);

  const found = keyspaces.find((ks) => ks.name === "types_test");
  t.truthy(found);
  t.true(found.tables.includes("all_types"));
});

test("getTable returns table metadata", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const table = session.getTable("types_test", "all_types");
  t.is(table.name, "all_types");
  t.deepEqual(table.partitionKey, ["id"]);
  t.true(table.columns.length > 0);

  const idCol = table.columns.find((c) => c.name === "id");
  t.truthy(idCol);
});

test("getTable throws for non-existent table", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  t.throws(() => session.getTable("types_test", "nonexistent"), {
    message: /not found/,
  });
});

test("refreshMetadata completes without error", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await t.notThrowsAsync(() => session.refreshMetadata());
});
