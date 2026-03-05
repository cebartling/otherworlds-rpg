import SwiftUI

/// Screen showing all campaign runs.
struct CampaignRunListView: View {
    @State private var viewModel: CampaignRunListViewModel

    private let endpoint: SessionEndpoint

    init(client: HTTPClientProtocol) {
        let ep = SessionEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: CampaignRunListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.campaignRuns.isEmpty {
                    LoadingView(message: "Loading sessions...")
                } else if viewModel.campaignRuns.isEmpty {
                    ContentUnavailableView(
                        "No Sessions",
                        systemImage: "play.slash",
                        description: Text("No campaign runs found.")
                    )
                } else {
                    campaignRunList
                }
            }
            .navigationTitle("Sessions")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadCampaignRuns()
            }
            .refreshable {
                await viewModel.loadCampaignRuns()
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

    private var campaignRunList: some View {
        List {
            ForEach(viewModel.campaignRuns) { run in
                NavigationLink(value: run.runId) {
                    CampaignRunRowView(campaignRun: run)
                }
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        Task { await viewModel.archiveCampaignRun(id: run.runId) }
                    } label: {
                        Label("Archive", systemImage: "archivebox")
                    }
                }
                .listRowBackground(Theme.surface)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.surface)
        .navigationDestination(for: UUID.self) { runId in
            CampaignRunDetailView(runId: runId, endpoint: endpoint)
        }
    }
}
