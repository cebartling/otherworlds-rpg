import SwiftUI

/// Detail screen for a single campaign run — shows checkpoints and commands.
struct CampaignRunDetailView: View {
    @State private var viewModel: CampaignRunDetailViewModel

    init(runId: UUID, endpoint: SessionEndpoint) {
        self._viewModel = State(
            initialValue: CampaignRunDetailViewModel(
                runId: runId,
                endpoint: endpoint
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.campaignRun == nil {
                LoadingView(message: "Loading session...")
            } else if let run = viewModel.campaignRun {
                campaignRunContent(run)
            } else {
                ContentUnavailableView(
                    "Session Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("Could not load this session.")
                )
            }
        }
        .background(Theme.surface)
        .navigationTitle("Session")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await viewModel.loadCampaignRun()
        }
        .refreshable {
            await viewModel.loadCampaignRun()
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

    private func campaignRunContent(_ run: CampaignRunDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text(run.runId.uuidString.prefix(8))
                    .font(.title2)
                    .foregroundStyle(Theme.accent)

                if let campaignId = run.campaignId {
                    Text("Campaign: \(campaignId.uuidString.prefix(8))")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                }

                Divider()
                    .overlay(Theme.border)

                // Checkpoints
                if run.checkpointIds.isEmpty {
                    Text("No checkpoints")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                } else {
                    Text("Checkpoints")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    ForEach(run.checkpointIds, id: \.self) { checkpointId in
                        HStack {
                            Label(
                                String(checkpointId.uuidString.prefix(8)),
                                systemImage: "flag.fill"
                            )
                            .font(.subheadline)
                            .foregroundStyle(Theme.text)
                            Spacer()
                            Button("Branch") {
                                Task {
                                    await viewModel.branchTimeline(
                                        fromCheckpointId: checkpointId,
                                        newRunId: UUID()
                                    )
                                }
                            }
                            .buttonStyle(.bordered)
                            .tint(Theme.accent)
                        }
                    }
                }

                Divider()
                    .overlay(Theme.border)

                Button("Create Checkpoint") {
                    Task { await viewModel.createCheckpoint(checkpointId: UUID()) }
                }
                .buttonStyle(.borderedProminent)
                .tint(Theme.accent)

                // Version
                HStack {
                    Spacer()
                    Text("Version \(run.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
