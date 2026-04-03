import { CorsaUtils } from "@corsa-bind/node";

import { isMain } from "../shared.ts";

export function runUnsafeTypeFlowExample() {
  return {
    assignmentIntoString: CorsaUtils.isUnsafeAssignment({
      sourceTypeTexts: ["Set<any>"],
      targetTypeTexts: ["Set<string>"],
    }),
    assignmentIntoUnknown: CorsaUtils.isUnsafeAssignment({
      sourceTypeTexts: ["any"],
      targetTypeTexts: ["unknown"],
    }),
    promiseReturn: CorsaUtils.isUnsafeReturn({
      sourceTypeTexts: ["Promise<any>"],
      targetTypeTexts: ["Promise<string>"],
    }),
  };
}

if (isMain(import.meta.url)) {
  console.log(JSON.stringify(runUnsafeTypeFlowExample(), null, 2));
}
