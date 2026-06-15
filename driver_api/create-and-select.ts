import { Cluster } from "../index.js";

const SCYLLA_NODE = process.env.SCYLLA_NODE || "127.0.0.1:9042";

async function main() {
  const cluster = new Cluster({
    nodes: [SCYLLA_NODE],
  });

  const session = await cluster.connect();

  await session.execute(
    "CREATE KEYSPACE IF NOT EXISTS driver_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }"
  );

  await session.execute("USE driver_test");

  await session.execute(`
    CREATE TABLE IF NOT EXISTS users (
      name text,
      age int,
      email text,
      PRIMARY KEY (name)
    )
  `);

  type User = {
    name: string;
    age: number;
    email: string;
  };

  const users: User[] = [
    { name: "John", age: 30, email: "john@doe.com" },
    { name: "Jane", age: 25, email: "jane@doe.com" },
  ];

  for (const user of users) {
    await session.execute(
      "INSERT INTO users (name, age, email) VALUES (?, ?, ?)",
      [user.name, user.age, user.email]
    );
  }

  const result = await session.execute("SELECT name, age, email FROM users");
  const rows = result.rows as User[];

  for (const { name, age, email } of rows) {
    console.log(`${name} is ${age} years old and has email ${email}`);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
