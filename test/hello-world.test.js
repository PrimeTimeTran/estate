// hello-world.test.js
const { tmpdir } = require('os');
const path = require('path');
const fs = require('fs');
const { assert, afterEach, beforeEach, describe, it } = require('assert');

const tempDir = path.join(tmpdir(), 'hello-world-test');
const testFilePath = path.join(tempDir, 'hello-world.md');

beforeEach(() => {
  fs.rmSync(testFilePath, { force: true });
});

afterEach(() => {
  fs.rmSync(testFilePath, { force: true });
});

describe('hello-world.cjs tests', () => {
  it('writes input to file', () => {
    const input = 'test input';
    const args = [path.join(process.cwd(), 'hello-world.cjs'), input];

    // Execute the CLI script
    const child = require('child_process').spawn('node', args, { stdio: 'pipe' });

    let output = '';
    child.stdout.on('data', data => {
      output += data;
    });

    child.stderr.on('data', data => {
      output += data;
    });

    child.on('close', () => {
      assert.ok(fs.existsSync(testFilePath), 'File should exist');
      const fileContent = fs.readFileSync(testFilePath, 'utf-8');
      assert.strictEqual(fileContent, input, 'File content should match input');
    });
  });
});
