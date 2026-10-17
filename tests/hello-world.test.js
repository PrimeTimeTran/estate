import { describe, it, expect, afterEach } from "vitest";
import { createRequire } from "node:module";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const require = createRequire(import.meta.url);
const execFilePromise = promisify(execFile);

const testDir = path.dirname(fileURLToPath(import.meta.url));
const estateDir = path.resolve(testDir, "..");

const scriptPath = path.resolve(testDir, "../hello-world.cjs");
const outputPath = path.resolve(testDir, "../hello-world.md");

afterEach(async () => {
  await rm(outputPath, { force: true });
});

describe("hello-world.cjs tests", () => {
  it("writes input to hello-world.md", async () => {
    await execFilePromise("node", [scriptPath, "test"]);

    const fileContent = await readFile(outputPath, "utf8");
    expect(fileContent).toBe("test");
  });

  it("handles empty input", async () => {
    await execFilePromise("node", [scriptPath, ""]);

    const fileContent = await readFile(outputPath, "utf8");
    expect(fileContent).toBe("");
  });

  it("outputs an error when no input is provided", async () => {
    const result = await execFilePromise("node", [scriptPath], { cwd: estateDir }).catch(
      (error) => error,
    );

    expect(result.stderr).toContain("Usage: node hello-world.cjs <string>");
    expect(result.code).toBe(1);
  });
});
