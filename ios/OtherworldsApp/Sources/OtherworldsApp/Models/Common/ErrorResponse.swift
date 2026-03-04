import Foundation

/// Structured error response from the Otherworlds backend.
///
/// Matches the JSON shape `{ "error": "...", "message": "..." }`.
struct ErrorResponse: Codable, Equatable, Sendable {
    let error: String
    let message: String
}
