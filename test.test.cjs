const fs = require('fs');
const path = require('path');
const assert = require('assert');

// Helper function to write content to a file
function writeMarkdownContent(input, filePath) {
  fs.writeFileSync(filePath, input);
}

// Test case: Basic content writing
describe('Markdown Content Writer', function() {
  it('should write the input to the file', function() {
    // Create temporary directory
    const tempDir = path.join(__dirname, 'temp');
    fs.mkdirSync(tempDir, { recursive: true });
    const filePath = path.join(tempDir, 'output.md');
    
    // Test input
    const input = 'Hello World';
    writeMarkdownContent(input, filePath);
    
    // Verify content matches input
    const content = fs.readFileSync(filePath, 'utf-8');
    assert.strictEqual(content, input);
    
    // Clean up
    fs.unlinkSync(filePath);
    fs.rmdirSync(tempDir);
  });
});

// Test case: Special characters handling
describe('Special Character Handling', function() {
  it('should preserve newline characters', function() {
    const tempDir = path.join(__dirname, 'temp');
    fs.mkdirSync(tempDir, { recursive: true });
    const filePath = path.join(tempDir, 'output.md');
    
    const input = 'New\nLine';
    writeMarkdownContent(input, filePath);
    
    const content = fs.readFileSync(filePath, 'utf-8');
    assert.strictEqual(content, input);
    
    fs.unlinkSync(filePath);
    fs.rmdirSync(tempDir);
  });
});

// Test case: Empty input handling
describe('Empty Input Handling', function() {
  it('should write empty string when input is empty', function() {
    const tempDir = path.join(__dirname, 'temp');
    fs.mkdirSync(tempDir, { recursive: true });
    const filePath = path.join(tempDir, 'output.md');
    
    const input = '';
    writeMarkdownContent(input, filePath);
    
    const content = fs.readFileSync(filePath, 'utf-8');
    assert.strictEqual(content, input);
    
    fs.unlinkSync(filePath);
    fs.rmdirSync(tempDir);
  });
});