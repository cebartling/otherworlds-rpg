//! Observability — structured logging via os.Logger

import os

/// Centralized logger instances for the Otherworlds app.
///
/// Each category maps to a logical subsystem boundary, making it easy
/// to filter logs in Console.app or Instruments.
enum AppLogger {
    /// HTTP request/response logging.
    static let api = Logger(subsystem: "com.otherworlds.app", category: "api")

    /// View model and UI lifecycle logging.
    static let ui = Logger(subsystem: "com.otherworlds.app", category: "ui")

    /// General-purpose logging for anything that doesn't fit above.
    static let general = Logger(subsystem: "com.otherworlds.app", category: "general")
}
