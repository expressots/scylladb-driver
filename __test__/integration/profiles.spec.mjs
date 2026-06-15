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

test("connect with execution profile (consistency local_one)", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    executionProfile: {
      consistency: "local_one",
      requestTimeoutMs: 10000,
      retryPolicy: "default",
    },
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

test("connect with fallthrough retry policy", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    executionProfile: {
      consistency: "one",
      retryPolicy: "fallthrough",
    },
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

test("connect with speculative execution", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    executionProfile: {
      consistency: "one",
      speculativeExecution: {
        maxRetryCount: 2,
        retryIntervalMs: 100,
      },
    },
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

test("invalid retry policy name errors", async (t) => {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
    executionProfile: {
      retryPolicy: "invalid_policy_name",
    },
  });
  await t.throwsAsync(() => cluster.connect(), {
    message: /Unknown retry policy/,
  });
});
