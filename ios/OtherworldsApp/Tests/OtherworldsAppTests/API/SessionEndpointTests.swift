import Foundation
import Testing

@testable import OtherworldsApp

@Suite("SessionEndpoint — correct paths and bodies via mock")
struct SessionEndpointTests {

    private func makeEndpoint() -> (SessionEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = SessionEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listCampaignRuns_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        mock.nextResult = .success("[]".data(using: .utf8)!)

        let _: [CampaignRunSummary] = try await endpoint.listCampaignRuns()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/session")
    }

    @Test func getCampaignRun_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let runId = UUID()
        let json = """
            {"run_id":"\(runId)","campaign_id":null,"checkpoint_ids":[],"version":1}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getCampaignRun(id: runId)

        #expect(mock.calls[0].path == "/api/v1/session/\(runId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func startCampaignRun_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = StartCampaignRunRequest(campaignId: nil)
        let _ = try await endpoint.startCampaignRun(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/session/start")
    }

    @Test func createCheckpoint_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = CreateCheckpointRequest(runId: UUID(), checkpointId: UUID())
        let _ = try await endpoint.createCheckpoint(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/session/create-checkpoint")
    }

    @Test func branchTimeline_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = BranchTimelineRequest(
            runId: UUID(),
            fromCheckpointId: UUID(),
            newRunId: UUID()
        )
        let _ = try await endpoint.branchTimeline(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/session/branch-timeline")
    }

    @Test func archiveCampaignRun_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let runId = UUID()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveCampaignRun(id: runId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/session/\(runId)")
    }
}
