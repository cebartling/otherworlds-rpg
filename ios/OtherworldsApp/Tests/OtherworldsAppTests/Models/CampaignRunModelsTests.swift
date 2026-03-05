import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CampaignRun Models — Codable round-trips")
struct CampaignRunModelsTests {

    @Test func campaignRunSummary_decodesFromSnakeCaseJSON() throws {
        let runId = UUID()
        let campaignId = UUID()
        let json = """
            {"run_id":"\(runId)","campaign_id":"\(campaignId)","checkpoint_count":3,"version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CampaignRunSummary.self, from: json)

        #expect(decoded.runId == runId)
        #expect(decoded.campaignId == campaignId)
        #expect(decoded.checkpointCount == 3)
        #expect(decoded.version == 1)
        #expect(decoded.id == runId)
    }

    @Test func campaignRunSummary_decodesNullCampaignId() throws {
        let runId = UUID()
        let json = """
            {"run_id":"\(runId)","campaign_id":null,"checkpoint_count":0,"version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CampaignRunSummary.self, from: json)

        #expect(decoded.campaignId == nil)
    }

    @Test func campaignRunDetail_decodesCheckpointIds() throws {
        let runId = UUID()
        let cpId = UUID()
        let json = """
            {"run_id":"\(runId)","campaign_id":null,"checkpoint_ids":["\(cpId)"],"version":2}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CampaignRunDetail.self, from: json)

        #expect(decoded.checkpointIds == [cpId])
    }

    @Test func startCampaignRunRequest_encodesToSnakeCase() throws {
        let campaignId = UUID()
        let request = StartCampaignRunRequest(campaignId: campaignId)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"campaign_id\""))
    }

    @Test func createCheckpointRequest_encodesToSnakeCase() throws {
        let request = CreateCheckpointRequest(runId: UUID(), checkpointId: UUID())
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"run_id\""))
        #expect(json.contains("\"checkpoint_id\""))
    }

    @Test func branchTimelineRequest_encodesToSnakeCase() throws {
        let request = BranchTimelineRequest(
            runId: UUID(),
            fromCheckpointId: UUID(),
            newRunId: UUID()
        )
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"from_checkpoint_id\""))
        #expect(json.contains("\"new_run_id\""))
    }
}
