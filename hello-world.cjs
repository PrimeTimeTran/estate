#!/usr/bin/env node
const fs = require('fs');
const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Usage: node hello-world.cjs <string>');
  process.exit(1);
}
const input = args[0];
fs.writeFileSync('hello-world.md', input);
