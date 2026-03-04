import Foundation
import Testing

@testable import OtherworldsApp

@Suite("HTTPClient — URL construction, headers, error parsing")
struct HTTPClientTests {

    // MARK: - MockHTTPClient basics

    @Test func mockClient_recordsGetCall() async throws {
        let mock = MockHTTPClient()
        let json = """
            {"status":"ok","version":"1.0.0"}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let result: HealthResponse = try await mock.get(path: "/health", correlationId: "abc-123")

        #expect(result.status == "ok")
        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/health")
        #expect(mock.calls[0].correlationId == "abc-123")
    }

    @Test func mockClient_recordsPostCall() async throws {
        let mock = MockHTTPClient()
        let eventId = UUID()
        let json = """
            {"event_ids":["\(eventId.uuidString)"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        struct TestBody: Encodable, Sendable {
            let sessionId: UUID
        }
        let body = TestBody(sessionId: UUID())

        let result: CommandResponse = try await mock.post(
            path: "/api/v1/narrative/advance-beat",
            body: body,
            correlationId: nil
        )

        #expect(result.eventIds == [eventId])
        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/narrative/advance-beat")
        #expect(mock.calls[0].body != nil)
    }

    @Test func mockClient_recordsDeleteCall() async throws {
        let mock = MockHTTPClient()
        let eventId = UUID()
        let json = """
            {"event_ids":["\(eventId.uuidString)"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let result: CommandResponse = try await mock.delete(
            path: "/api/v1/narrative/session-1",
            correlationId: nil
        )

        #expect(result.eventIds == [eventId])
        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "DELETE")
    }

    @Test func mockClient_throwsConfiguredError() async {
        let mock = MockHTTPClient()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await #expect(throws: APIError.self) {
            let _: HealthResponse = try await mock.get(path: "/health", correlationId: nil)
        }
    }

    @Test func mockClient_postBodyEncodesSnakeCase() async throws {
        let mock = MockHTTPClient()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        struct CamelBody: Encodable, Sendable {
            let sessionId: UUID
            let choiceIndex: Int
        }
        let body = CamelBody(sessionId: UUID(), choiceIndex: 2)

        let _: CommandResponse = try await mock.post(
            path: "/test",
            body: body,
            correlationId: nil
        )

        let bodyData = mock.calls[0].body!
        let bodyDict = try JSONSerialization.jsonObject(with: bodyData) as! [String: Any]
        #expect(bodyDict.keys.contains("session_id"))
        #expect(bodyDict.keys.contains("choice_index"))
    }
}
