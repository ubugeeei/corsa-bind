import native from "../index.js";
function fromJson(value) {
    return JSON.parse(value);
}
function toJson(value) {
    return JSON.stringify(value ?? null);
}
export class CorsaApiClient {
    #inner;
    constructor(inner) {
        this.#inner = inner;
    }
    static spawn(options) {
        return new CorsaApiClient(native.CorsaApiClient.spawn(toJson(options)));
    }
    initialize() {
        return fromJson(this.#inner.initializeJson());
    }
    parseConfigFile(file) {
        return fromJson(this.#inner.parseConfigFileJson(file));
    }
    updateSnapshot(params) {
        return fromJson(this.#inner.updateSnapshotJson(params ? toJson(params) : undefined));
    }
    getSourceFile(snapshot, project, file) {
        return this.#inner.getSourceFile(snapshot, project, file) ?? null;
    }
    getStringType(snapshot, project) {
        return fromJson(this.#inner.getStringTypeJson(snapshot, project));
    }
    typeToString(snapshot, project, typeHandle, location, flags) {
        return this.#inner.typeToString(snapshot, project, typeHandle, location, flags);
    }
    callJson(method, params) {
        return fromJson(this.#inner.callJson(method, params ? toJson(params) : undefined));
    }
    callBinary(method, params) {
        return this.#inner.callBinary(method, params ? toJson(params) : undefined) ?? null;
    }
    releaseHandle(handle) {
        this.#inner.releaseHandle(handle);
    }
    close() {
        this.#inner.close();
    }
}
export class CorsaVirtualDocument {
    #inner;
    constructor(inner) {
        this.#inner = inner;
    }
    static untitled(path, languageId, text) {
        return new CorsaVirtualDocument(native.CorsaVirtualDocument.untitled(path, languageId, text));
    }
    static inMemory(authority, path, languageId, text) {
        return new CorsaVirtualDocument(native.CorsaVirtualDocument.inMemory(authority, path, languageId, text));
    }
    get uri() {
        return this.#inner.uri;
    }
    get languageId() {
        return this.#inner.languageId;
    }
    get version() {
        return this.#inner.version;
    }
    get text() {
        return this.#inner.text;
    }
    state() {
        return fromJson(this.#inner.stateJson());
    }
    replace(text) {
        this.#inner.replace(text);
    }
    applyChanges(changes) {
        return fromJson(this.#inner.applyChangesJson(toJson(changes)));
    }
}
export class CorsaDistributedOrchestrator {
    #inner;
    constructor(nodeIds) {
        this.#inner = new native.CorsaDistributedOrchestrator(nodeIds);
    }
    campaign(nodeId) {
        return this.#inner.campaign(nodeId);
    }
    leaderId() {
        return this.#inner.leaderId() ?? undefined;
    }
    state() {
        const value = this.#inner.stateJson();
        return value ? fromJson(value) : undefined;
    }
    nodeState(nodeId) {
        const value = this.#inner.nodeStateJson(nodeId);
        return value ? fromJson(value) : undefined;
    }
    document(nodeId, uri) {
        const value = this.#inner.documentJson(nodeId, uri);
        return value ? fromJson(value) : undefined;
    }
    openVirtualDocument(document) {
        return fromJson(this.#inner.openVirtualDocumentJson(this.requireLeader(), toJson(document)));
    }
    changeVirtualDocument(uri, changes) {
        return fromJson(this.#inner.changeVirtualDocumentJson(this.requireLeader(), uri, toJson(changes)));
    }
    closeVirtualDocument(uri) {
        this.#inner.closeVirtualDocument(this.requireLeader(), uri);
    }
    requireLeader() {
        const leaderId = this.leaderId();
        if (!leaderId) {
            throw new Error("raft leader has not been elected");
        }
        return leaderId;
    }
}
export const binding = native;
export const version = native.version;
//# sourceMappingURL=index.js.map