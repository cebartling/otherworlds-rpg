import Foundation
import Testing

@testable import OtherworldsApp

@Suite("InventoryListViewModel — loading, success, error states")
@MainActor
struct InventoryListViewModelTests {

    private func makeViewModel() -> (InventoryListViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = InventoryEndpoint(client: mock)
        let vm = InventoryListViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadInventories_success_populatesInventories() async {
        let (vm, mock) = makeViewModel()
        let inventoryId = UUID()
        let json = """
            [{"inventory_id":"\(inventoryId)","item_count":3,"version":1}]
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadInventories()

        #expect(vm.inventories.count == 1)
        #expect(vm.inventories[0].inventoryId == inventoryId)
        #expect(vm.inventories[0].itemCount == 3)
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadInventories_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadInventories()

        #expect(vm.inventories.isEmpty)
        #expect(vm.isLoading == false)
        #expect(vm.error == .httpError(statusCode: 500))
    }

    @Test func archiveInventory_success_reloadsInventories() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.archiveInventory(id: UUID())

        #expect(mock.calls.count == 2) // delete + reload
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))
        await vm.loadInventories()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
