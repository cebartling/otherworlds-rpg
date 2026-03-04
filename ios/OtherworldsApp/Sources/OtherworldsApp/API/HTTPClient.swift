import Foundation

/// Production HTTP client backed by `URLSession`.
///
/// Mirrors the web client's `apiFetch<T>` pattern: prepends base URL,
/// sets JSON headers, forwards correlation IDs, parses error responses.
final class HTTPClient: HTTPClientProtocol, @unchecked Sendable {
    private let baseURL: URL
    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder

    init(baseURL: URL, session: URLSession = .shared) {
        self.baseURL = baseURL
        self.session = session

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        self.decoder = decoder

        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        self.encoder = encoder
    }

    func get<T: Decodable & Sendable>(path: String, correlationId: String? = nil) async throws -> T {
        try await perform(method: "GET", path: path, correlationId: correlationId)
    }

    func post<T: Decodable & Sendable, B: Encodable & Sendable>(
        path: String,
        body: B,
        correlationId: String? = nil
    ) async throws -> T {
        let bodyData = try encoder.encode(body)
        return try await perform(method: "POST", path: path, body: bodyData, correlationId: correlationId)
    }

    func delete<T: Decodable & Sendable>(path: String, correlationId: String? = nil) async throws -> T {
        try await perform(method: "DELETE", path: path, correlationId: correlationId)
    }

    private func perform<T: Decodable>(
        method: String,
        path: String,
        body: Data? = nil,
        correlationId: String?
    ) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw APIError.network("Invalid URL path: \(path)")
        }

        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        if let correlationId {
            request.setValue(correlationId, forHTTPHeaderField: "X-Correlation-ID")
        }

        if let body {
            request.httpBody = body
        }

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.network(error.localizedDescription)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.network("Response is not HTTP")
        }

        guard (200...299).contains(httpResponse.statusCode) else {
            if let errorResponse = try? decoder.decode(ErrorResponse.self, from: data) {
                throw APIError.server(
                    statusCode: httpResponse.statusCode,
                    error: errorResponse.error,
                    message: errorResponse.message
                )
            }
            throw APIError.httpError(statusCode: httpResponse.statusCode)
        }

        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw APIError.decodingFailed(error.localizedDescription)
        }
    }
}
