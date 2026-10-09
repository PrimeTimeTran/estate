// hello-world.cjs
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Usage: node hello-world.cjs <input>');
  process.exit(1);
}

const input = args[0];
const filePath = path.join(process.cwd(), 'hello-world.md');

try {
  fs.writeFileSync(filePath, input);
  console.log();
} catch (err) {
  console.error('Error writing file:', err.message);
  process.exit(1);
}
