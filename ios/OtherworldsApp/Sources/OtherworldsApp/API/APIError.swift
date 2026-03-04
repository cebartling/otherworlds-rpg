import Foundation

/// Errors that can occur when communicating with the Otherworlds backend API.
enum APIError: LocalizedError, Equatable, Sendable {
    /// The server returned a structured error response.
    case server(statusCode: Int, error: String, message: String)
    /// The server returned a non-OK status without a parseable body.
    case httpError(statusCode: Int)
    /// A network-level failure (no response received).
    case network(String)
    /// The response body could not be decoded into the expected type.
    case decodingFailed(String)

    var errorDescription: String? {
        switch self {
        case let .server(statusCode, _, message):
            "Server error \(statusCode): \(message)"
        case let .httpError(statusCode):
            "HTTP error \(statusCode)"
        case let .network(message):
            "Network error: \(message)"
        case let .decodingFailed(message):
            "Decoding failed: \(message)"
        }
    }
}
