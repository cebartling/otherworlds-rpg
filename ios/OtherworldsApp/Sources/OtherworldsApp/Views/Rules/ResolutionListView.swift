import SwiftUI

/// Screen showing all resolutions.
struct ResolutionListView: View {
    @State private var viewModel: ResolutionListViewModel

    private let endpoint: RulesEndpoint

    init(client: HTTPClientProtocol) {
        let ep = RulesEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: ResolutionListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.resolutions.isEmpty {
                    LoadingView(message: "Loading resolutions...")
                } else if viewModel.resolutions.isEmpty {
                    ContentUnavailableView(
                        "No Resolutions",
                        systemImage: "dice",
                        description: Text("No resolutions found.")
                    )
                } else {
                    resolutionList
                }
            }
            .navigationTitle("Rules")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadResolutions()
            }
            .refreshable {
                await viewModel.loadResolutions()
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

    private var resolutionList: some View {
        List {
            ForEach(viewModel.resolutions) { resolution in
                NavigationLink(value: resolution.resolutionId) {
                    ResolutionRowView(resolution: resolution)
                }
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        Task { await viewModel.archiveResolution(id: resolution.resolutionId) }
                    } label: {
                        Label("Archive", systemImage: "archivebox")
                    }
                }
                .listRowBackground(Theme.surface)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.surface)
        .navigationDestination(for: UUID.self) { resolutionId in
            ResolutionDetailView(resolutionId: resolutionId, endpoint: endpoint)
        }
    }
}
