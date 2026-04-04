import Testing
@testable import CorsaUtils

@Test func smoke() async throws {
    #expect(CorsaUtils.classifyTypeText("Promise<string> | null") == "nullish")
    #expect(CorsaUtils.splitTypeText("string | Promise<any>") == ["string", "Promise<any>"])
    #expect(CorsaUtils.isErrorLikeTypeTexts(["TypeError"]))
    #expect(CorsaUtils.hasUnsafeAnyFlow(["Promise<any>"], targetTexts: ["Promise<string>"]))
    let document = try CorsaVirtualDocument.untitled(path: "/demo.ts", languageID: "typescript", text: "const value = 1;")
    try document.splice(startLine: 0, startCharacter: 14, endLine: 0, endCharacter: 15, text: "2")
    #expect(document.text == "const value = 2;")
    #expect(document.version == 2)
}
