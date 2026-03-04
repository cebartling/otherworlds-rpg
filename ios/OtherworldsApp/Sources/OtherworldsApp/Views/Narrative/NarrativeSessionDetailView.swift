import SwiftUI

/// Play screen for a single narrative session — shows current scene and choices.
struct NarrativeSessionDetailView: View {
    @State private var viewModel: NarrativeSessionDetailViewModel

    init(sessionId: UUID, client: NarrativeEndpoint) {
        self._viewModel = State(
            initialValue: NarrativeSessionDetailViewModel(
                sessionId: sessionId,
                endpoint: client
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.session == nil {
                LoadingView(message: "Loading session...")
            } else if let session = viewModel.session {
                sessionContent(session)
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
            await viewModel.loadSession()
        }
        .refreshable {
            await viewModel.loadSession()
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

    private func sessionContent(_ session: NarrativeSessionView) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Scene info
                if let sceneId = session.currentSceneId {
                    Label(sceneId, systemImage: "book.pages")
                        .font(.title2)
                        .foregroundStyle(Theme.accent)
                } else {
                    Text("No active scene")
                        .font(.title2)
                        .foregroundStyle(Theme.textMuted)
                }

                // Scene history
                if !session.sceneHistory.isEmpty {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Scene History")
                            .font(.caption)
                            .foregroundStyle(Theme.textMuted)
                        Text(session.sceneHistory.joined(separator: " > "))
                            .font(.subheadline)
                            .foregroundStyle(Theme.text)
                    }
                }

                Divider()
                    .overlay(Theme.border)

                // Choices
                if session.activeChoiceOptions.isEmpty {
                    Button("Advance Beat") {
                        Task { await viewModel.advanceBeat() }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(Theme.accent)
                } else {
                    Text("Choose your path:")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    ForEach(
                        Array(session.activeChoiceOptions.enumerated()),
                        id: \.offset
                    ) { index, choice in
                        ChoiceButtonView(index: index + 1, label: choice.label) {
                            Task {
                                let target = TargetSceneRequest(
                                    sceneId: choice.targetSceneId,
                                    narrativeText: "",
                                    choices: [],
                                    npcRefs: nil
                                )
                                await viewModel.selectChoice(index: index, targetScene: target)
                            }
                        }
                    }
                }

                // Version
                HStack {
                    Spacer()
                    Text("Version \(session.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
