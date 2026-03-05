import SwiftUI

/// Screen showing all world snapshots.
struct WorldSnapshotListView: View {
    @State private var viewModel: WorldSnapshotListViewModel

    private let endpoint: WorldStateEndpoint

    init(client: HTTPClientProtocol) {
        let ep = WorldStateEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: WorldSnapshotListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.worldSnapshots.isEmpty {
                    LoadingView(message: "Loading world snapshots...")
                } else if viewModel.worldSnapshots.isEmpty {
                    ContentUnavailableView(
                        "No World Snapshots",
                        systemImage: "globe.badge.chevron.backward",
                        description: Text("No world snapshots found.")
                    )
                } else {
                    worldSnapshotList
                }
            }
            .navigationTitle("World")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadWorldSnapshots()
            }
            .refreshable {
                await viewModel.loadWorldSnapshots()
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

    private var worldSnapshotList: some View {
        List {
            ForEach(viewModel.worldSnapshots) { snapshot in
                NavigationLink(value: snapshot.worldId) {
                    WorldSnapshotRowView(worldSnapshot: snapshot)
                }
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        Task { await viewModel.archiveWorldSnapshot(id: snapshot.worldId) }
                    } label: {
                        Label("Archive", systemImage: "archivebox")
                    }
                }
                .listRowBackground(Theme.surface)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.surface)
        .navigationDestination(for: UUID.self) { worldId in
            WorldSnapshotDetailView(worldId: worldId, endpoint: endpoint)
        }
    }
}
