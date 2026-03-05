import SwiftUI

/// Screen showing all inventories.
struct InventoryListView: View {
    @State private var viewModel: InventoryListViewModel

    private let endpoint: InventoryEndpoint

    init(client: HTTPClientProtocol) {
        let ep = InventoryEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: InventoryListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.inventories.isEmpty {
                    LoadingView(message: "Loading inventories...")
                } else if viewModel.inventories.isEmpty {
                    ContentUnavailableView(
                        "No Inventories",
                        systemImage: "bag.slash",
                        description: Text("No inventories found.")
                    )
                } else {
                    inventoryList
                }
            }
            .navigationTitle("Inventory")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadInventories()
            }
            .refreshable {
                await viewModel.loadInventories()
            }
            .overlay(alignment: .top) {
                if let error = viewModel.error {
                    ErrorBannerView(
                        message: error.localizedDescription,
                        onDismiss: { viewModel.dismissError() }
                    )
                }
            }
        }
    }

    private var inventoryList: some View {
        List {
            ForEach(viewModel.inventories) { inventory in
                NavigationLink(value: inventory.inventoryId) {
                    InventoryRowView(inventory: inventory)
                }
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        Task { await viewModel.archiveInventory(id: inventory.inventoryId) }
                    } label: {
                        Label("Archive", systemImage: "archivebox")
                    }
                }
                .listRowBackground(Theme.surface)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.surface)
        .navigationDestination(for: UUID.self) { inventoryId in
            InventoryDetailView(inventoryId: inventoryId, endpoint: endpoint)
        }
    }
}
