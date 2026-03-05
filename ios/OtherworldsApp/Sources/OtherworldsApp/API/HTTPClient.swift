import Foundation
import os

/// Production HTTP client backed by `URLSession`.
///
/// Mirrors the web client's `apiFetch<T>` pattern: prepends base URL,
/// sets JSON headers, forwards correlation IDs, parses error responses.
/// Includes structured logging via `os.Logger` and Instruments signposts.
final class HTTPClient: HTTPClientProtocol, @unchecked Sendable {
    private let baseURL: URL
    private let session: URLSession
    private let decoder: JSONDecoder
    private let encoder: JSONEncoder
    private let signposter = OSSignposter(logger: AppLogger.api)

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
        let resolvedCorrelationId = correlationId ?? CorrelationId.generate()
        let startTime = CFAbsoluteTimeGetCurrent()
        let signpostId = signposter.makeSignpostID()
        let signpostState = signposter.beginInterval("HTTP Request", id: signpostId, "\(method) \(path)")

        AppLogger.api.info("\(method) \(path) [correlation: \(resolvedCorrelationId)]")

        guard let url = URL(string: path, relativeTo: baseURL) else {
            signposter.endInterval("HTTP Request", signpostState)
            AppLogger.api.error("\(method) \(path) failed: invalid URL [correlation: \(resolvedCorrelationId)]")
            throw APIError.network("Invalid URL path: \(path)")
        }

        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.setValue(resolvedCorrelationId, forHTTPHeaderField: "X-Correlation-ID")

        if let body {
            request.httpBody = body
        }

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            let duration = CFAbsoluteTimeGetCurrent() - startTime
            signposter.endInterval("HTTP Request", signpostState)
            AppLogger.api.error(
                "\(method) \(path) failed: \(error.localizedDescription) (\(String(format: "%.0f", duration * 1000))ms) [correlation: \(resolvedCorrelationId)]"
            )
            throw APIError.network(error.localizedDescription)
        }

        guard let httpResponse = response as? HTTPURLResponse else {
            signposter.endInterval("HTTP Request", signpostState)
            AppLogger.api.error(
                "\(method) \(path) failed: response is not HTTP [correlation: \(resolvedCorrelationId)]"
            )
            throw APIError.network("Response is not HTTP")
        }

        let duration = CFAbsoluteTimeGetCurrent() - startTime
        let durationMs = String(format: "%.0f", duration * 1000)
        signposter.endInterval("HTTP Request", signpostState)

        guard (200...299).contains(httpResponse.statusCode) else {
            AppLogger.api.error(
                "\(method) \(path) -> \(httpResponse.statusCode) (\(durationMs)ms) [correlation: \(resolvedCorrelationId)]"
            )
            if let errorResponse = try? decoder.decode(ErrorResponse.self, from: data) {
                throw APIError.server(
                    statusCode: httpResponse.statusCode,
                    error: errorResponse.error,
                    message: errorResponse.message
                )
            }
            throw APIError.httpError(statusCode: httpResponse.statusCode)
        }

        AppLogger.api.info(
            "\(method) \(path) -> \(httpResponse.statusCode) (\(durationMs)ms) [correlation: \(resolvedCorrelationId)]"
        )

        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            AppLogger.api.error(
                "\(method) \(path) decoding failed: \(error.localizedDescription) [correlation: \(resolvedCorrelationId)]"
            )
            throw APIError.decodingFailed(error.localizedDescription)
        }
    }
}
