import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import type { TypeAwareParserOptions } from "typescript-oxlint";

const examplesDir = dirname(fileURLToPath(import.meta.url));
const workspaceRoot = resolve(examplesDir, "../..");

export function createExampleParserOptions(): TypeAwareParserOptions {
  return {
    projectService: {
      allowDefaultProject: ["*.ts", "*.tsx"],
    },
    tsconfigRootDir: workspaceRoot,
    tsgo: {
      executable: resolve(workspaceRoot, ".cache/tsgo"),
      cwd: workspaceRoot,
      mode: "msgpack",
      requestTimeoutMs: 30_000,
    },
  };
}

export function createExampleSettings() {
  return {
    typescriptOxlint: {
      parserOptions: createExampleParserOptions(),
    },
  };
}
