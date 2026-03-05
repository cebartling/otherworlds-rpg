import Foundation
import Testing

@testable import OtherworldsApp

@Suite("InventoryDetailViewModel — loading, commands, error states")
@MainActor
struct InventoryDetailViewModelTests {

    private func makeViewModel(inventoryId: UUID = UUID()) -> (InventoryDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = InventoryEndpoint(client: mock)
        let vm = InventoryDetailViewModel(inventoryId: inventoryId, endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadInventory_success_populatesInventory() async {
        let inventoryId = UUID()
        let (vm, mock) = makeViewModel(inventoryId: inventoryId)
        let json = """
            {"inventory_id":"\(inventoryId)","items":["Sword","Shield"],"version":2}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadInventory()

        #expect(vm.inventory != nil)
        #expect(vm.inventory?.items == ["Sword", "Shield"])
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadInventory_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))

        await vm.loadInventory()

        #expect(vm.inventory == nil)
        #expect(vm.error == .httpError(statusCode: 404))
    }

    @Test func addItem_success_reloadsInventory() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.addItem(name: "Potion")

        #expect(mock.calls.count == 2) // post + reload
        #expect(mock.calls[0].method == "POST")
    }

    @Test func removeItem_success_reloadsInventory() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.removeItem(name: "Sword")

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func equipItem_success_reloadsInventory() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.equipItem(name: "Shield")

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))
        await vm.loadInventory()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
