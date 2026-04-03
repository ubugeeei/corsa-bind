import native from "../index.js";
import type { ApiClientOptions, ConfigResponse, InitializeResponse, TypeResponse, UpdateSnapshotParams, UpdateSnapshotResponse, VirtualChange, VirtualDocumentState } from "./types";
export declare class CorsaApiClient {
    #private;
    private constructor();
    static spawn(options: ApiClientOptions): CorsaApiClient;
    initialize(): InitializeResponse;
    parseConfigFile(file: string): ConfigResponse;
    updateSnapshot(params?: UpdateSnapshotParams): UpdateSnapshotResponse;
    getSourceFile(snapshot: string, project: string, file: string): Uint8Array | null;
    getStringType(snapshot: string, project: string): TypeResponse;
    typeToString(snapshot: string, project: string, typeHandle: string, location?: string, flags?: number): string;
    callJson<T>(method: string, params?: unknown): T;
    callBinary(method: string, params?: unknown): Uint8Array | null;
    releaseHandle(handle: string): void;
    close(): void;
}
export declare class CorsaVirtualDocument {
    #private;
    private constructor();
    static untitled(path: string, languageId: string, text: string): CorsaVirtualDocument;
    static inMemory(authority: string, path: string, languageId: string, text: string): CorsaVirtualDocument;
    get uri(): string;
    get languageId(): string;
    get version(): number;
    get text(): string;
    state(): VirtualDocumentState;
    replace(text: string): void;
    applyChanges(changes: VirtualChange[]): unknown[];
}
export declare class CorsaDistributedOrchestrator {
    #private;
    constructor(nodeIds: string[]);
    campaign(nodeId: string): number;
    leaderId(): string | undefined;
    state<T>(): T | undefined;
    nodeState<T>(nodeId: string): T | undefined;
    document(nodeId: string, uri: string): VirtualDocumentState | undefined;
    openVirtualDocument(document: VirtualDocumentState): VirtualDocumentState;
    changeVirtualDocument(uri: string, changes: VirtualChange[]): VirtualDocumentState;
    closeVirtualDocument(uri: string): void;
    private requireLeader;
}
export declare const binding: typeof native;
export declare const version: typeof native.version;
export type * from "./types";
