import Foundation
import Testing

@testable import OtherworldsApp

@Suite("InventoryEndpoint — correct paths and bodies via mock")
struct InventoryEndpointTests {

    private func makeEndpoint() -> (InventoryEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = InventoryEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listInventories_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        mock.nextResult = .success("[]".data(using: .utf8)!)

        let _: [InventorySummary] = try await endpoint.listInventories()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/inventory")
    }

    @Test func getInventory_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let inventoryId = UUID()
        let json = """
            {"inventory_id":"\(inventoryId)","items":[],"version":1}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getInventory(id: inventoryId)

        #expect(mock.calls[0].path == "/api/v1/inventory/\(inventoryId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func addItem_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = AddItemRequest(inventoryId: UUID(), itemName: "Sword")
        let _ = try await endpoint.addItem(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/inventory/add-item")
    }

    @Test func removeItem_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = RemoveItemRequest(inventoryId: UUID(), itemName: "Sword")
        let _ = try await endpoint.removeItem(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/inventory/remove-item")
    }

    @Test func equipItem_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = EquipItemRequest(inventoryId: UUID(), itemName: "Shield")
        let _ = try await endpoint.equipItem(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/inventory/equip-item")
    }

    @Test func archiveInventory_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let inventoryId = UUID()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveInventory(id: inventoryId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/inventory/\(inventoryId)")
    }
}
