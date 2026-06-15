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
      "CREATE KEYSPACE IF NOT EXISTS prepared_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
    );
    await session.useKeyspace("prepared_test");
    await session.execute(`
      CREATE TABLE IF NOT EXISTS items (
        id int PRIMARY KEY,
        name text,
        price double
      )
    `);
    t.context.session = session;
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("prepare and execute an INSERT", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const prepared = await session.prepare(
    "INSERT INTO items (id, name, price) VALUES (?, ?, ?)"
  );
  await prepared.execute([1, "Widget", 9.99]);
  await prepared.execute([2, "Gadget", 19.99]);

  const result = await session.execute("SELECT * FROM items WHERE id = ?", [1]);
  t.is(result.rows[0].name, "Widget");
  t.is(result.rows[0].price, 9.99);
});

test("prepare and execute a SELECT", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await session.execute(
    "INSERT INTO items (id, name, price) VALUES (?, ?, ?)",
    [2, "Gadget", 19.99]
  );

  const prepared = await session.prepare(
    "SELECT name, price FROM items WHERE id = ?"
  );
  const result = await prepared.execute([2]);
  t.is(result.rowLength, 1);
  t.is(result.rows[0].name, "Gadget");
});

test("prepared statement getQuery returns original query", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const prepared = await session.prepare(
    "SELECT * FROM items WHERE id = ?"
  );
  t.is(prepared.getQuery(), "SELECT * FROM items WHERE id = ?");
});

test("prepared statement with consistency option", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const prepared = await session.prepare(
    "SELECT * FROM items WHERE id = ?"
  );
  const result = await prepared.execute([1], { consistency: "one" });
  t.is(result.rowLength, 1);
});

test("prepare with invalid query returns error", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await t.throwsAsync(
    () => session.prepare("SELECT * FROM nonexistent_table_xyz"),
    { message: /Failed to prepare statement/ }
  );
});
