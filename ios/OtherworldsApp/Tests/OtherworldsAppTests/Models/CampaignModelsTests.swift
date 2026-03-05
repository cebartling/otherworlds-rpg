import Foundation
import Testing

@testable import OtherworldsApp

@Suite("Campaign Models — Codable round-trips")
struct CampaignModelsTests {

    @Test func campaignSummary_decodesFromSnakeCaseJSON() throws {
        let campaignId = UUID()
        let json = """
            {"campaign_id":"\(campaignId)","version_hash":"abc123","phase":"ingested","version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CampaignSummary.self, from: json)

        #expect(decoded.campaignId == campaignId)
        #expect(decoded.versionHash == "abc123")
        #expect(decoded.phase == "ingested")
        #expect(decoded.version == 1)
        #expect(decoded.id == campaignId)
    }

    @Test func campaignSummary_decodesNullHash() throws {
        let campaignId = UUID()
        let json = """
            {"campaign_id":"\(campaignId)","version_hash":null,"phase":"created","version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CampaignSummary.self, from: json)

        #expect(decoded.versionHash == nil)
    }

    @Test func campaignDetail_decodesAllFields() throws {
        let campaignId = UUID()
        let json = """
            {"campaign_id":"\(campaignId)","version_hash":"abc123","source":"# Campaign","compiled_data":"compiled","phase":"compiled","version":3}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CampaignDetail.self, from: json)

        #expect(decoded.source == "# Campaign")
        #expect(decoded.compiledData == "compiled")
        #expect(decoded.phase == "compiled")
    }

    @Test func ingestCampaignRequest_encodesToSnakeCase() throws {
        let campaignId = UUID()
        let request = IngestCampaignRequest(campaignId: campaignId, source: "# Test")
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"campaign_id\""))
        #expect(json.contains("\"source\":\"# Test\""))
    }

    @Test func validateCampaignRequest_encodesToSnakeCase() throws {
        let campaignId = UUID()
        let request = ValidateCampaignRequest(campaignId: campaignId)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"campaign_id\""))
    }

    @Test func compileCampaignRequest_encodesToSnakeCase() throws {
        let campaignId = UUID()
        let request = CompileCampaignRequest(campaignId: campaignId)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"campaign_id\""))
    }
}
