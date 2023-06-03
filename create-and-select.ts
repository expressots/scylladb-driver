import { Cluster } from "./index";

async function main() {
  const cluster = new Cluster({
    nodes: ["127.0.0.1:9042"],
  });

  const session = await cluster.connect();

  async function createKeyspace() {
    await session.execute(
      `CREATE KEYSPACE IF NOT EXISTS driver_test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }`
    );
  }

  // Create a simple table for user <name, age, email>
  async function createTable() {
    await session.execute(`
    CREATE TABLE IF NOT EXISTS users (
      name text,
      age int,
      email text,
      PRIMARY KEY (name)
    );
  `);
  }

  // Insert a new user if the user does not exist
  type User = {
    name: string;
    age: number;
    email: string;
  };

  async function insertUser(user: User) {
    await session.execute(`
    INSERT INTO users (name, age, email)
    VALUES ('${user.name}', ${user.age}, '${user.email}')
    IF NOT EXISTS;
  `);
  }

  async function selectUser(name: string) {
    const result = await session.execute(`
    SELECT * FROM users WHERE name = '${name}';
  `);
    return result[0];
  }

  async function selectAllUsers(): Promise<User[]> {
    const result = await session.execute(`
    SELECT * FROM users;
  `);
    return result;
  }

  await createKeyspace();
  await session.execute("USE driver_test");
  await createTable();

  [
    {
      name: "John",
      age: 30,
      email: "john@doe.com",
    },
    {
      name: "Jane",
      age: 25,
      email: "jane@doe.com",
    },
  ].forEach(insertUser);

  const users = await selectAllUsers();

  console.log(users);

  users.forEach(({ name, age, email }) => {
    console.log(`${name} is ${age} years old and has email ${email}`);
  });
}

main();
