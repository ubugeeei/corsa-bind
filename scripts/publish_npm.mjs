import {
  publishPackedTarball,
  sleep,
  typescriptOxlintPackage,
  withStagedNodeBindingPackages,
} from "./npm_release_utils.mjs";

const delayMs = Number(process.env.NPM_PUBLISH_DELAY_MS ?? "10000");
const distTag = process.env.NPM_DIST_TAG?.trim() || undefined;
const artifactsDir = process.env.NAPI_ARTIFACTS_DIR?.trim() || undefined;
const requireAllTargets = process.env.NAPI_REQUIRE_ALL_TARGETS === "1";

await withStagedNodeBindingPackages(
  { artifactsDir, requireAllTargets },
  async ({ binaryPackages, rootPackage }) => {
    const releasePackages = [...binaryPackages, rootPackage, typescriptOxlintPackage];

    for (const [index, pkg] of releasePackages.entries()) {
      publishPackedTarball(pkg, { tag: distTag });
      if (index + 1 < releasePackages.length && delayMs > 0) {
        await sleep(delayMs);
      }
    }
  },
);
