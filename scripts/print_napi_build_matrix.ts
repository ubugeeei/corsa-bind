import { getNodeBindingBuildMatrix } from "./npm_release_utils.ts";

process.stdout.write(JSON.stringify({ include: getNodeBindingBuildMatrix() }));
