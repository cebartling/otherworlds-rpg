import SwiftUI

/// Detail screen for a single campaign — shows phase, source, compiled data, and commands.
struct CampaignDetailView: View {
    @State private var viewModel: CampaignDetailViewModel

    init(campaignId: UUID, endpoint: ContentEndpoint) {
        self._viewModel = State(
            initialValue: CampaignDetailViewModel(
                campaignId: campaignId,
                endpoint: endpoint
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.campaign == nil {
                LoadingView(message: "Loading campaign...")
            } else if let campaign = viewModel.campaign {
                campaignContent(campaign)
            } else {
                ContentUnavailableView(
                    "Campaign Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("Could not load this campaign.")
                )
            }
        }
        .background(Theme.surface)
        .navigationTitle("Campaign")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await viewModel.loadCampaign()
        }
        .refreshable {
            await viewModel.loadCampaign()
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

    private func campaignContent(_ campaign: CampaignDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text(campaign.campaignId.uuidString.prefix(8))
                    .font(.title2)
                    .foregroundStyle(Theme.accent)

                // Phase badge
                HStack {
                    Text("Phase:")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                    Text(campaign.phase)
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(Theme.surfaceAlt)
                        .clipShape(Capsule())
                        .foregroundStyle(Theme.text)
                }

                if let hash = campaign.versionHash {
                    Text("Hash: \(hash.prefix(16))")
                        .font(.caption)
                        .foregroundStyle(Theme.textMuted)
                }

                Divider()
                    .overlay(Theme.border)

                // Source
                Text("Source")
                    .font(.headline)
                    .foregroundStyle(Theme.accent)

                if let source = campaign.source {
                    Text(source)
                        .font(.subheadline)
                        .foregroundStyle(Theme.text)
                } else {
                    Text("No source data")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                }

                // Compiled
                if let compiled = campaign.compiledData {
                    Divider()
                        .overlay(Theme.border)

                    Text("Compiled Data")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    Text(compiled)
                        .font(.subheadline)
                        .foregroundStyle(Theme.text)
                }

                Divider()
                    .overlay(Theme.border)

                // Phase-appropriate buttons
                if campaign.phase == "ingested" {
                    Button("Validate") {
                        Task { await viewModel.validateCampaign() }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(Theme.accent)
                } else if campaign.phase == "validated" {
                    Button("Compile") {
                        Task { await viewModel.compileCampaign() }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(Theme.accent)
                } else {
                    Button("Ingest") {
                        Task { await viewModel.ingestCampaign(source: "sample source") }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(Theme.accent)
                }

                // Version
                HStack {
                    Spacer()
                    Text("Version \(campaign.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
