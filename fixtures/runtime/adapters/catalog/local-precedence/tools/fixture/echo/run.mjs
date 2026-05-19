#!/usr/bin/env node
const message = process.env.RUNX_INPUT_MESSAGE || "";
process.stdout.write(`local:${message}`);
