import type { CorsaRemoteTransport } from "./client.ts";
import type { CorsaUtilsLike, UnsafeTypeFlowInput } from "./types.ts";

export interface RemoteCorsaUtilsOptions {
  versionMethod?: string;
  isUnsafeAssignmentMethod?: string;
  isUnsafeReturnMethod?: string;
}

export const DEFAULT_REMOTE_CORSA_UTIL_METHODS = Object.freeze({
  versionMethod: "corsa.utils.version",
  isUnsafeAssignmentMethod: "corsa.utils.isUnsafeAssignment",
  isUnsafeReturnMethod: "corsa.utils.isUnsafeReturn",
});

export function createRemoteCorsaUtils(
  transport: Pick<CorsaRemoteTransport, "requestJson">,
  options: RemoteCorsaUtilsOptions = {},
): CorsaUtilsLike<Promise<string>, Promise<boolean>> {
  const methods = {
    ...DEFAULT_REMOTE_CORSA_UTIL_METHODS,
    ...options,
  };

  return Object.freeze({
    version(): Promise<string> {
      return transport.requestJson<string>(methods.versionMethod);
    },
    isUnsafeAssignment(input: UnsafeTypeFlowInput): Promise<boolean> {
      return transport.requestJson<boolean>(methods.isUnsafeAssignmentMethod, input);
    },
    isUnsafeReturn(input: UnsafeTypeFlowInput): Promise<boolean> {
      return transport.requestJson<boolean>(methods.isUnsafeReturnMethod, input);
    },
  });
}
