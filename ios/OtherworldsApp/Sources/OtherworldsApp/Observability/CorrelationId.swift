//! Observability — correlation ID generation

import Foundation

/// Generates unique correlation IDs for request tracing.
///
/// Correlation IDs flow from the iOS client through HTTP headers
/// to the backend, enabling end-to-end request tracing.
enum CorrelationId {
    /// Generate a new lowercase UUID suitable for use as a correlation ID.
    static func generate() -> String {
        UUID().uuidString.lowercased()
    }
}
