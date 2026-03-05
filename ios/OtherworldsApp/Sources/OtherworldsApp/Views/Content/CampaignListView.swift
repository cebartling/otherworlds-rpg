import SwiftUI

/// Screen showing all campaigns.
struct CampaignListView: View {
    @State private var viewModel: CampaignListViewModel

    private let endpoint: ContentEndpoint

    init(client: HTTPClientProtocol) {
        let ep = ContentEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: CampaignListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.campaigns.isEmpty {
                    LoadingView(message: "Loading campaigns...")
                } else if viewModel.campaigns.isEmpty {
                    ContentUnavailableView(
                        "No Campaigns",
                        systemImage: "doc.text.magnifyingglass",
                        description: Text("No campaigns found.")
                    )
                } else {
                    campaignList
                }
            }
            .navigationTitle("Content")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadCampaigns()
            }
            .refreshable {
                await viewModel.loadCampaigns()
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

    private var campaignList: some View {
        List {
            ForEach(viewModel.campaigns) { campaign in
                NavigationLink(value: campaign.campaignId) {
                    CampaignRowView(campaign: campaign)
                }
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        Task { await viewModel.archiveCampaign(id: campaign.campaignId) }
                    } label: {
                        Label("Archive", systemImage: "archivebox")
                    }
                }
                .listRowBackground(Theme.surface)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.surface)
        .navigationDestination(for: UUID.self) { campaignId in
            CampaignDetailView(campaignId: campaignId, endpoint: endpoint)
        }
    }
}
