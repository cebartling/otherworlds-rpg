import Foundation
import Testing

@testable import OtherworldsApp

@Suite("Inventory Models — Codable round-trips")
struct InventoryModelsTests {

    @Test func inventorySummary_decodesFromSnakeCaseJSON() throws {
        let inventoryId = UUID()
        let json = """
            {"inventory_id":"\(inventoryId)","item_count":5,"version":2}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(InventorySummary.self, from: json)

        #expect(decoded.inventoryId == inventoryId)
        #expect(decoded.itemCount == 5)
        #expect(decoded.version == 2)
        #expect(decoded.id == inventoryId)
    }

    @Test func inventoryDetail_decodesItemsArray() throws {
        let inventoryId = UUID()
        let json = """
            {"inventory_id":"\(inventoryId)","items":["Sword","Shield"],"version":3}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(InventoryDetail.self, from: json)

        #expect(decoded.inventoryId == inventoryId)
        #expect(decoded.items == ["Sword", "Shield"])
        #expect(decoded.version == 3)
    }

    @Test func addItemRequest_encodesToSnakeCase() throws {
        let inventoryId = UUID()
        let request = AddItemRequest(inventoryId: inventoryId, itemName: "Potion")
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"item_name\":\"Potion\""))
        #expect(json.contains("\"inventory_id\""))
    }

    @Test func removeItemRequest_encodesToSnakeCase() throws {
        let inventoryId = UUID()
        let request = RemoveItemRequest(inventoryId: inventoryId, itemName: "Sword")
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"item_name\":\"Sword\""))
    }

    @Test func equipItemRequest_encodesToSnakeCase() throws {
        let inventoryId = UUID()
        let request = EquipItemRequest(inventoryId: inventoryId, itemName: "Shield")
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"item_name\":\"Shield\""))
    }
}
