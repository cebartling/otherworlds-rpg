import Foundation

/// Abstraction over HTTP communication for testability.
///
/// Production code uses `HTTPClient`; tests inject `MockHTTPClient`.
protocol HTTPClientProtocol: Sendable {
    /// Perform a GET request and decode the response.
    func get<T: Decodable & Sendable>(path: String, correlationId: String?) async throws -> T

    /// Perform a POST request with a JSON body and decode the response.
    func post<T: Decodable & Sendable, B: Encodable & Sendable>(
        path: String,
        body: B,
        correlationId: String?
    ) async throws -> T

    /// Perform a DELETE request and decode the response.
    func delete<T: Decodable & Sendable>(path: String, correlationId: String?) async throws -> T
}
