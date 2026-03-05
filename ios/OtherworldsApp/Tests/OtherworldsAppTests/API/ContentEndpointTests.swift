import Foundation
import Testing

@testable import OtherworldsApp

@Suite("ContentEndpoint — correct paths and bodies via mock")
struct ContentEndpointTests {

    private func makeEndpoint() -> (ContentEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = ContentEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listCampaigns_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        mock.nextResult = .success("[]".data(using: .utf8)!)

        let _: [CampaignSummary] = try await endpoint.listCampaigns()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/content")
    }

    @Test func getCampaign_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let campaignId = UUID()
        let json = """
            {"campaign_id":"\(campaignId)","version_hash":null,"source":null,"compiled_data":null,"phase":"created","version":1}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getCampaign(id: campaignId)

        #expect(mock.calls[0].path == "/api/v1/content/\(campaignId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func ingestCampaign_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = IngestCampaignRequest(campaignId: UUID(), source: "# Test")
        let _ = try await endpoint.ingestCampaign(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/content/ingest")
    }

    @Test func validateCampaign_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = ValidateCampaignRequest(campaignId: UUID())
        let _ = try await endpoint.validateCampaign(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/content/validate")
    }

    @Test func compileCampaign_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = CompileCampaignRequest(campaignId: UUID())
        let _ = try await endpoint.compileCampaign(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/content/compile")
    }

    @Test func archiveCampaign_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let campaignId = UUID()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveCampaign(id: campaignId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/content/\(campaignId)")
    }
}
