import Foundation

/// Request to add an item to an inventory.
struct AddItemRequest: Codable, Equatable, Sendable {
    let inventoryId: UUID
    let itemName: String
}

/// Request to remove an item from an inventory.
struct RemoveItemRequest: Codable, Equatable, Sendable {
    let inventoryId: UUID
    let itemName: String
}

/// Request to equip an item in an inventory.
struct EquipItemRequest: Codable, Equatable, Sendable {
    let inventoryId: UUID
    let itemName: String
}
