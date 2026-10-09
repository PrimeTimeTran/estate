// This script reads the first positional argument and writes it to hello-world.md
const fs = require('fs');
const path = require('path');

if (process.argv.length < 3) {
  console.error('Usage: node hello-world.cjs <input>');
  process.exit(1);
}

const input = process.argv[2];
const filePath = path.resolve(path.join(__dirname, 'hello-world.md'));

try {
  fs.writeFileSync(filePath, input);
  console.log();
} catch (err) {
  console.error('Error writing file:', err.message);
  process.exit(1);
}