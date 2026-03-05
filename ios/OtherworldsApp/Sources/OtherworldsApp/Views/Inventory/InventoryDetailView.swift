import SwiftUI

/// Detail screen for a single inventory — shows items and commands.
struct InventoryDetailView: View {
    @State private var viewModel: InventoryDetailViewModel

    init(inventoryId: UUID, endpoint: InventoryEndpoint) {
        self._viewModel = State(
            initialValue: InventoryDetailViewModel(
                inventoryId: inventoryId,
                endpoint: endpoint
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.inventory == nil {
                LoadingView(message: "Loading inventory...")
            } else if let inventory = viewModel.inventory {
                inventoryContent(inventory)
            } else {
                ContentUnavailableView(
                    "Inventory Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("Could not load this inventory.")
                )
            }
        }
        .background(Theme.surface)
        .navigationTitle("Inventory")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await viewModel.loadInventory()
        }
        .refreshable {
            await viewModel.loadInventory()
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

    private func inventoryContent(_ inventory: InventoryDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text(inventory.inventoryId.uuidString.prefix(8))
                    .font(.title2)
                    .foregroundStyle(Theme.accent)

                Divider()
                    .overlay(Theme.border)

                // Items
                if inventory.items.isEmpty {
                    Text("No items")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                } else {
                    Text("Items")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    ForEach(inventory.items, id: \.self) { item in
                        HStack {
                            Text(item)
                                .font(.subheadline)
                                .foregroundStyle(Theme.text)
                            Spacer()
                            Button("Equip") {
                                Task { await viewModel.equipItem(name: item) }
                            }
                            .buttonStyle(.bordered)
                            .tint(Theme.accent)
                            Button("Remove") {
                                Task { await viewModel.removeItem(name: item) }
                            }
                            .buttonStyle(.bordered)
                            .tint(.red)
                        }
                    }
                }

                Divider()
                    .overlay(Theme.border)

                Button("Add Item") {
                    Task { await viewModel.addItem(name: "New Item") }
                }
                .buttonStyle(.borderedProminent)
                .tint(Theme.accent)

                // Version
                HStack {
                    Spacer()
                    Text("Version \(inventory.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
