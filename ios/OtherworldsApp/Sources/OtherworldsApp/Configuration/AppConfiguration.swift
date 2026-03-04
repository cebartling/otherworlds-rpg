import Foundation

/// Persists app-level configuration such as the backend API base URL.
@Observable
final class AppConfiguration {
    private static let baseURLKey = "api_base_url"
    private static let defaultBaseURL = "http://localhost:3000"

    var baseURLString: String {
        didSet {
            UserDefaults.standard.set(baseURLString, forKey: Self.baseURLKey)
        }
    }

    var baseURL: URL {
        URL(string: baseURLString) ?? URL(string: Self.defaultBaseURL)!
    }

    init() {
        self.baseURLString = UserDefaults.standard.string(forKey: Self.baseURLKey)
            ?? Self.defaultBaseURL
    }

    func makeHTTPClient() -> HTTPClient {
        HTTPClient(baseURL: baseURL)
    }
}
