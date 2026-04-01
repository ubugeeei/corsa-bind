import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  cpSync,
  mkdtempSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
const pnpmCommand = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

export const nodeBindingPackage = {
  name: "@tsgo-rs/node",
  path: resolve(rootDir, "npm/tsgo_rs_node"),
  access: "public",
};

export const typescriptOxlintPackage = {
  name: "typescript-oxlint",
  path: resolve(rootDir, "npm/typescript_oxlint"),
};

export const npmPackages = [nodeBindingPackage, typescriptOxlintPackage];

const defaultTargetTriples = [
  "x86_64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-gnu",
];

const sysToNodePlatform = {
  android: "android",
  darwin: "darwin",
  freebsd: "freebsd",
  linux: "linux",
  windows: "win32",
};

const cpuToNodeArch = {
  aarch64: "arm64",
  arm: "arm",
  armv7: "arm",
  i686: "ia32",
  riscv64: "riscv64",
  universal: "universal",
  x86_64: "x64",
};

function run(command, args, cwd = rootDir) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    env: process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function pickDefined(input, keys) {
  return keys.reduce((output, key) => {
    if (input[key] !== undefined) {
      output[key] = input[key];
    }
    return output;
  }, {});
}

export function parseTargetTriple(rawTriple) {
  const normalized = rawTriple.endsWith("eabi") ? `${rawTriple.slice(0, -4)}-eabi` : rawTriple;
  const parts = normalized.split("-");
  let cpu;
  let sys;
  let abi = null;

  if (parts.length === 4) {
    [cpu, , sys, abi = null] = parts;
  } else if (parts.length === 3) {
    [cpu, , sys] = parts;
  } else {
    [cpu, sys] = parts;
  }

  const platform = sysToNodePlatform[sys] ?? sys;
  const arch = cpuToNodeArch[cpu] ?? cpu;
  const target = {
    abi,
    arch,
    platform,
    platformArchABI: abi ? `${platform}-${arch}-${abi}` : `${platform}-${arch}`,
    raw: rawTriple,
  };

  if (abi === "gnu") {
    target.libc = "glibc";
  } else if (abi === "musl") {
    target.libc = "musl";
  }

  return target;
}

export function getNodeBindingTargets(
  packageJson = readJson(resolve(nodeBindingPackage.path, "package.json")),
) {
  const useDefaults = packageJson.napi?.triples?.defaults !== false;
  const additionalTargets = packageJson.napi?.triples?.additional ?? [];
  return [...(useDefaults ? defaultTargetTriples : []), ...additionalTargets].map(
    parseTargetTriple,
  );
}

export function createBinaryPackageManifest(rootManifest, version, target, binaryFileName) {
  const manifest = {
    ...pickDefined(rootManifest, [
      "author",
      "authors",
      "bugs",
      "description",
      "engines",
      "homepage",
      "keywords",
      "license",
      "publishConfig",
      "repository",
    ]),
    files: [binaryFileName],
    main: binaryFileName,
    name: `${rootManifest.name}-${target.platformArchABI}`,
    os: [target.platform],
    version,
  };

  if (target.arch !== "universal") {
    manifest.cpu = [target.arch];
  }

  if (target.libc) {
    manifest.libc = [target.libc];
  }

  return manifest;
}

export function createRootBindingPublishManifest(rootManifest, version, stagedTargets) {
  const manifest = {
    ...rootManifest,
    optionalDependencies: Object.fromEntries(
      stagedTargets.map((target) => [`${rootManifest.name}-${target.platformArchABI}`, version]),
    ),
    version,
  };

  if (Array.isArray(manifest.files)) {
    manifest.files = manifest.files.filter((entry) => entry !== "*.node");
  }

  return manifest;
}

function createBinaryPackageReadme(packageName, target) {
  return `# \`${packageName}-${target.platformArchABI}\`\n\nThis is the **${target.raw}** binary for \`${packageName}\`\n`;
}

function findFilesRecursive(directory) {
  if (!directory) {
    return [];
  }

  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      return findFilesRecursive(entryPath);
    }
    return [entryPath];
  });
}

function collectBindingArtifacts({ binaryName, searchRoots }) {
  const artifacts = new Map();

  for (const root of searchRoots) {
    for (const filePath of findFilesRecursive(root)) {
      const fileName = basename(filePath);
      if (!fileName.startsWith(`${binaryName}.`) || !fileName.endsWith(".node")) {
        continue;
      }

      const platformArchABI = fileName.slice(binaryName.length + 1, -".node".length);
      if (!artifacts.has(platformArchABI)) {
        artifacts.set(platformArchABI, {
          fileName,
          path: filePath,
          platformArchABI,
        });
      }
    }
  }

  return artifacts;
}

function copyRootBindingPackage(stagePath) {
  cpSync(nodeBindingPackage.path, stagePath, {
    filter(sourcePath) {
      if (sourcePath === nodeBindingPackage.path) {
        return true;
      }

      const relativePath = relative(nodeBindingPackage.path, sourcePath).replaceAll("\\", "/");
      if (relativePath.startsWith("npm/")) {
        return false;
      }
      if (relativePath.endsWith(".node")) {
        return false;
      }
      return true;
    },
    recursive: true,
  });
}

export function stageNodeBindingPackages({ artifactsDir, requireAllTargets = false } = {}) {
  const rootManifest = readJson(resolve(nodeBindingPackage.path, "package.json"));
  const version = rootManifest.version;
  const binaryName = rootManifest.napi?.name ?? "index";
  const configuredTargets = getNodeBindingTargets(rootManifest);
  const searchRoots = artifactsDir ? [resolve(rootDir, artifactsDir)] : [nodeBindingPackage.path];

  if (!artifactsDir || !requireAllTargets) {
    searchRoots.push(nodeBindingPackage.path);
  }

  const artifacts = collectBindingArtifacts({
    binaryName,
    searchRoots: [...new Set(searchRoots)],
  });
  const missingTargets = configuredTargets.filter(
    (target) => !artifacts.has(target.platformArchABI),
  );

  if (requireAllTargets && missingTargets.length > 0) {
    throw new Error(
      `Missing native binding artifacts for: ${missingTargets.map((target) => target.platformArchABI).join(", ")}`,
    );
  }

  const stagedTargets = configuredTargets.filter((target) => artifacts.has(target.platformArchABI));
  if (stagedTargets.length === 0) {
    throw new Error("No native binding artifacts were found for the Node release packages.");
  }

  const stageDir = mkdtempSync(resolve(tmpdir(), "tsgo-rs-npm-stage-"));
  const stageRootPackagePath = resolve(stageDir, "tsgo_rs_node");

  copyRootBindingPackage(stageRootPackagePath);

  const stagedRootManifest = createRootBindingPublishManifest(
    readJson(resolve(stageRootPackagePath, "package.json")),
    version,
    stagedTargets,
  );
  writeJson(resolve(stageRootPackagePath, "package.json"), stagedRootManifest);

  const binaryPackages = stagedTargets.map((target) => {
    const artifact = artifacts.get(target.platformArchABI);
    const packagePath = resolve(stageDir, "npm", target.platformArchABI);
    mkdirSync(packagePath, { recursive: true });
    writeJson(
      resolve(packagePath, "package.json"),
      createBinaryPackageManifest(stagedRootManifest, version, target, artifact.fileName),
    );
    writeFileSync(
      resolve(packagePath, "README.md"),
      createBinaryPackageReadme(stagedRootManifest.name, target),
      "utf8",
    );
    copyFileSync(artifact.path, resolve(packagePath, artifact.fileName));
    return {
      access: nodeBindingPackage.access,
      name: `${stagedRootManifest.name}-${target.platformArchABI}`,
      path: packagePath,
    };
  });

  return {
    binaryPackages,
    cleanup() {
      rmSync(stageDir, { recursive: true, force: true });
    },
    missingTargets,
    rootPackage: {
      ...nodeBindingPackage,
      path: stageRootPackagePath,
    },
    stagedTargets,
  };
}

export async function withStagedNodeBindingPackages(options, callback) {
  const staged = stageNodeBindingPackages(options);
  try {
    return await callback(staged);
  } finally {
    staged.cleanup();
  }
}

export function withPackedTarball(pkg, callback) {
  const packDir = mkdtempSync(resolve(tmpdir(), "tsgo-rs-npm-pack-"));
  try {
    run(pnpmCommand, ["pack", "--pack-destination", packDir], pkg.path);
    const tarballName = readdirSync(packDir).find((entry) => entry.endsWith(".tgz"));
    if (!tarballName) {
      throw new Error(`Failed to pack npm tarball for ${pkg.name}`);
    }
    return callback(resolve(packDir, tarballName));
  } finally {
    rmSync(packDir, { recursive: true, force: true });
  }
}

export function publishPackedTarball(pkg, { dryRun = false, tag } = {}) {
  return withPackedTarball(pkg, (tarballPath) => {
    const args = ["publish", tarballPath];
    if (pkg.access) {
      args.push("--access", pkg.access);
    }
    if (tag) {
      args.push("--tag", tag);
    }
    if (dryRun) {
      args.push("--dry-run");
    }
    run(npmCommand, args, rootDir);
  });
}

export function sleep(ms) {
  return new Promise((resolveSleep) => setTimeout(resolveSleep, ms));
}
