// ../../node_modules/.bin/vitest run --globals test.test.cjs
const fs = require('fs');

const args = process.argv.slice(2);
if (args.length !== 1) {
  console.error('Error: Please provide exactly one argument.');
  process.exit(1);
}
const content = args[0];
fs.writeFileSync('output.md', content);


