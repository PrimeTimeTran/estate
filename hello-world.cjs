const fs = require(fs);
const args = process.argv.slice(2);
if (args.length === 0) {
  console.error(Error: Missing argument);
  process.exit(1);
}
const content = args[0];
fs.writeFileSync(hello-world.md, content);
console.log(File written);
