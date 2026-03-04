import Foundation
import Testing

@testable import OtherworldsApp

@Suite("Common Models — Codable round-trips")
struct CommonModelsTests {

    // MARK: - ErrorResponse

    @Test func errorResponse_decodesFromJSON() throws {
        let json = """
            {"error":"not_found","message":"Session not found"}
            """.data(using: .utf8)!

        let decoded = try JSONDecoder().decode(ErrorResponse.self, from: json)

        #expect(decoded.error == "not_found")
        #expect(decoded.message == "Session not found")
    }

    @Test func errorResponse_encodesAndDecodes() throws {
        let original = ErrorResponse(error: "conflict", message: "Version mismatch")
        let data = try JSONEncoder().encode(original)
        let decoded = try JSONDecoder().decode(ErrorResponse.self, from: data)

        #expect(decoded == original)
    }

    // MARK: - CommandResponse

    @Test func commandResponse_decodesFromSnakeCaseJSON() throws {
        let id1 = UUID()
        let id2 = UUID()
        let json = """
            {"event_ids":["\(id1.uuidString)","\(id2.uuidString)"]}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CommandResponse.self, from: json)

        #expect(decoded.eventIds == [id1, id2])
    }

    @Test func commandResponse_decodesEmptyEventIds() throws {
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CommandResponse.self, from: json)

        #expect(decoded.eventIds.isEmpty)
    }

    // MARK: - HealthResponse

    @Test func healthResponse_decodesFromJSON() throws {
        let json = """
            {"status":"ok","version":"0.1.0"}
            """.data(using: .utf8)!

        let decoded = try JSONDecoder().decode(HealthResponse.self, from: json)

        #expect(decoded.status == "ok")
        #expect(decoded.version == "0.1.0")
    }

    @Test func healthResponse_encodesAndDecodes() throws {
        let original = HealthResponse(status: "ok", version: "1.2.3")
        let data = try JSONEncoder().encode(original)
        let decoded = try JSONDecoder().decode(HealthResponse.self, from: data)

        #expect(decoded == original)
    }

    // MARK: - APIError

    @Test func apiError_serverDescription() {
        let error = APIError.server(statusCode: 404, error: "not_found", message: "Session not found")
        #expect(error.errorDescription == "Server error 404: Session not found")
    }

    @Test func apiError_httpErrorDescription() {
        let error = APIError.httpError(statusCode: 500)
        #expect(error.errorDescription == "HTTP error 500")
    }

    @Test func apiError_networkDescription() {
        let error = APIError.network("The Internet connection appears to be offline.")
        #expect(error.errorDescription == "Network error: The Internet connection appears to be offline.")
    }

    @Test func apiError_decodingFailedDescription() {
        let error = APIError.decodingFailed("Missing key 'session_id'")
        #expect(error.errorDescription == "Decoding failed: Missing key 'session_id'")
    }

    @Test func apiError_equatable() {
        let a = APIError.httpError(statusCode: 403)
        let b = APIError.httpError(statusCode: 403)
        let c = APIError.httpError(statusCode: 404)

        #expect(a == b)
        #expect(a != c)
    }
}
