import test from "ava";

import { Uuid } from "../index.js";

test("Create a random UUID", (t) => {
  const uuid = Uuid.randomUuid();
  t.is(uuid.toString().length, 36);
});

test("Create a UUID from a string", (t) => {
  const uuid = Uuid.fromString("123e4567-e89b-12d3-a456-426614174000");
  t.is(uuid.toString(), "123e4567-e89b-12d3-a456-426614174000");
});
