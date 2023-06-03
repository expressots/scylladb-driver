import { Cluster } from "./index";

async function main() {
  const cluster = new Cluster({
    nodes: ["127.0.0.1:9042"],
  });

  const session = await cluster.connect();

  console.log(await session.execute("SELECT table_name FROM system_schema.scylla_tables LIMIT 5"));
}

main();
