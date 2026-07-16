#!/usr/bin/env node

import { fail, run } from "../lib/client.mjs";

try {
  await run("read");
} catch (error) {
  fail(error instanceof Error ? error.message : String(error));
}
