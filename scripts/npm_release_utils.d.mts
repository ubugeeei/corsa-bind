export interface NodeBindingTarget {
  abi: string | null;
  arch: string;
  libc?: string;
  platform: string;
  platformArchABI: string;
  raw: string;
}

export interface PublishablePackage {
  access?: string;
  name: string;
  path: string;
}

export interface StageNodeBindingPackagesOptions {
  artifactsDir?: string;
  requireAllTargets?: boolean;
}

export interface StagedNodeBindingPackages {
  binaryPackages: PublishablePackage[];
  cleanup(): void;
  missingTargets: NodeBindingTarget[];
  rootPackage: PublishablePackage;
  stagedTargets: NodeBindingTarget[];
}

export const rootDir: string;
export const nodeBindingPackage: PublishablePackage;
export const typescriptOxlintPackage: PublishablePackage;
export const npmPackages: PublishablePackage[];

export function parseTargetTriple(rawTriple: string): NodeBindingTarget;
export function getNodeBindingTargets(packageJson?: Record<string, unknown>): NodeBindingTarget[];
export function createBinaryPackageManifest(
  rootManifest: Record<string, unknown>,
  version: string,
  target: NodeBindingTarget,
  binaryFileName: string,
): Record<string, unknown>;
export function createRootBindingPublishManifest(
  rootManifest: Record<string, unknown>,
  version: string,
  stagedTargets: NodeBindingTarget[],
): Record<string, unknown>;
export function stageNodeBindingPackages(
  options?: StageNodeBindingPackagesOptions,
): StagedNodeBindingPackages;
export function withStagedNodeBindingPackages<T>(
  options: StageNodeBindingPackagesOptions,
  callback: (staged: StagedNodeBindingPackages) => Promise<T> | T,
): Promise<T>;
export function withPackedTarball<T>(
  pkg: PublishablePackage,
  callback: (tarballPath: string) => T,
): T;
export function publishPackedTarball(
  pkg: PublishablePackage,
  options?: { dryRun?: boolean; tag?: string },
): void;
export function sleep(ms: number): Promise<void>;
