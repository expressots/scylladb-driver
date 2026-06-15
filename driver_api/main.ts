import { Cluster } from "../index.js";

const SCYLLA_NODE = process.env.SCYLLA_NODE || "127.0.0.1:9042";

async function main() {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
  });

  const session = await cluster.connect();
  const result = await session.execute(
    "SELECT release_version FROM system.local"
  );

  console.log(result.rows);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
