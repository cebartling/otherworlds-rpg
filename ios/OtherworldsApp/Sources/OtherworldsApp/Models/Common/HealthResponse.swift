import Foundation

/// Response from GET /health.
struct HealthResponse: Codable, Equatable, Sendable {
    let status: String
    let version: String
}
