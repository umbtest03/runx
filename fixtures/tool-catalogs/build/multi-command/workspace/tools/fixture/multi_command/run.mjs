#!/usr/bin/env node
const modeIndex = process.argv.indexOf("--mode");
const mode = modeIndex === -1 ? "default" : process.argv[modeIndex + 1];
process.stdout.write(JSON.stringify({ mode }));
