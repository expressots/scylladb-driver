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
      "CREATE KEYSPACE IF NOT EXISTS batch_paging_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
    );
    await session.useKeyspace("batch_paging_test");
    await session.execute(`
      CREATE TABLE IF NOT EXISTS batch_items (
        id int PRIMARY KEY,
        name text
      )
    `);
    await session.execute(`
      CREATE TABLE IF NOT EXISTS paging_items (
        id int PRIMARY KEY,
        value text
      )
    `);
    await session.execute(`
      CREATE TABLE IF NOT EXISTS lwt_items (
        id int PRIMARY KEY,
        name text
      )
    `);
    t.context.session = session;
  } catch (error) {
    if (!skipIfNoScylla(t, error)) throw error;
  }
});

test("batch inserts multiple rows", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const batch = session.batch("logged");
  batch.add({ query: "INSERT INTO batch_items (id, name) VALUES (?, ?)", params: [1, "one"] });
  batch.add({ query: "INSERT INTO batch_items (id, name) VALUES (?, ?)", params: [2, "two"] });
  batch.add({ query: "INSERT INTO batch_items (id, name) VALUES (?, ?)", params: [3, "three"] });
  await batch.execute();

  const result = await session.execute("SELECT id, name FROM batch_items");
  t.true(result.rowLength >= 3);
});

test("unlogged batch works", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  const batch = session.batch("unlogged");
  batch.add({ query: "INSERT INTO batch_items (id, name) VALUES (?, ?)", params: [10, "ten"] });
  batch.add({ query: "INSERT INTO batch_items (id, name) VALUES (?, ?)", params: [11, "eleven"] });
  await batch.execute();

  const result = await session.execute("SELECT id FROM batch_items WHERE id = ?", [10]);
  t.is(result.rowLength, 1);
});

test("paging fetches pages individually", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  // Insert enough rows to paginate
  for (let i = 0; i < 25; i++) {
    await session.execute(
      "INSERT INTO paging_items (id, value) VALUES (?, ?)",
      [i, `val-${i}`]
    );
  }

  // Fetch first page of 10
  const page1 = await session.querySinglePage(
    "SELECT id, value FROM paging_items",
    null,
    10,
    null
  );
  t.is(page1.rowLength, 10);
  t.truthy(page1.nextPageToken);

  // Fetch second page
  const page2 = await session.querySinglePage(
    "SELECT id, value FROM paging_items",
    null,
    10,
    page1.nextPageToken
  );
  t.is(page2.rowLength, 10);
});

test("LWT INSERT IF NOT EXISTS returns wasApplied", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await session.execute("DELETE FROM lwt_items WHERE id = ?", [100]);

  // First insert should succeed
  const result1 = await session.execute(
    "INSERT INTO lwt_items (id, name) VALUES (?, ?) IF NOT EXISTS",
    [100, "first"]
  );
  t.is(result1.wasApplied, true);

  // Second insert of same key should fail
  const result2 = await session.execute(
    "INSERT INTO lwt_items (id, name) VALUES (?, ?) IF NOT EXISTS",
    [100, "second"]
  );
  t.is(result2.wasApplied, false);
});

test("LWT conditional UPDATE returns wasApplied", async (t) => {
  const { session } = t.context;
  if (!session) return t.pass("Skipped");

  await session.execute("INSERT INTO lwt_items (id, name) VALUES (?, ?)", [200, "original"]);

  // Correct condition
  const result1 = await session.execute(
    "UPDATE lwt_items SET name = ? WHERE id = ? IF name = ?",
    ["updated", 200, "original"]
  );
  t.is(result1.wasApplied, true);

  // Wrong condition
  const result2 = await session.execute(
    "UPDATE lwt_items SET name = ? WHERE id = ? IF name = ?",
    ["again", 200, "nonexistent"]
  );
  t.is(result2.wasApplied, false);
});
