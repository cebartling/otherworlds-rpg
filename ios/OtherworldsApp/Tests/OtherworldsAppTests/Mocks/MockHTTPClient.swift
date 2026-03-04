import Foundation

@testable import OtherworldsApp

/// Test double for `HTTPClientProtocol`.
///
/// Records calls and returns pre-configured responses or throws errors.
final class MockHTTPClient: HTTPClientProtocol, @unchecked Sendable {

    struct RecordedCall: Sendable {
        let method: String
        let path: String
        let correlationId: String?
        let body: Data?
    }

    private(set) var calls: [RecordedCall] = []

    /// The result to return from the next call.
    /// Set to `.success(data)` with JSON-encoded data, or `.failure(error)`.
    var nextResult: Result<Data, APIError> = .success(Data())

    private let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()

    private let encoder: JSONEncoder = {
        let e = JSONEncoder()
        e.keyEncodingStrategy = .convertToSnakeCase
        return e
    }()

    func get<T: Decodable & Sendable>(path: String, correlationId: String?) async throws -> T {
        calls.append(RecordedCall(method: "GET", path: path, correlationId: correlationId, body: nil))
        return try decodeResult()
    }

    func post<T: Decodable & Sendable, B: Encodable & Sendable>(
        path: String,
        body: B,
        correlationId: String?
    ) async throws -> T {
        let bodyData = try encoder.encode(body)
        calls.append(RecordedCall(method: "POST", path: path, correlationId: correlationId, body: bodyData))
        return try decodeResult()
    }

    func delete<T: Decodable & Sendable>(path: String, correlationId: String?) async throws -> T {
        calls.append(RecordedCall(method: "DELETE", path: path, correlationId: correlationId, body: nil))
        return try decodeResult()
    }

    private func decodeResult<T: Decodable>() throws -> T {
        switch nextResult {
        case let .success(data):
            return try decoder.decode(T.self, from: data)
        case let .failure(error):
            throw error
        }
    }
}
