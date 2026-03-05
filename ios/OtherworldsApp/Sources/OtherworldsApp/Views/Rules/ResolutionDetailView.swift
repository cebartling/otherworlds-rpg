import SwiftUI

/// Detail screen for a single resolution — shows phase, intent, check result, effects, and commands.
struct ResolutionDetailView: View {
    @State private var viewModel: ResolutionDetailViewModel

    init(resolutionId: UUID, endpoint: RulesEndpoint) {
        self._viewModel = State(
            initialValue: ResolutionDetailViewModel(
                resolutionId: resolutionId,
                endpoint: endpoint
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.resolution == nil {
                LoadingView(message: "Loading resolution...")
            } else if let resolution = viewModel.resolution {
                resolutionContent(resolution)
            } else {
                ContentUnavailableView(
                    "Resolution Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("Could not load this resolution.")
                )
            }
        }
        .background(Theme.surface)
        .navigationTitle("Resolution")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await viewModel.loadResolution()
        }
        .refreshable {
            await viewModel.loadResolution()
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

    private func resolutionContent(_ resolution: ResolutionDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text(resolution.resolutionId.uuidString.prefix(8))
                    .font(.title2)
                    .foregroundStyle(Theme.accent)

                // Phase badge
                HStack {
                    Text("Phase:")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                    Text(resolution.phase)
                        .font(.subheadline)
                        .fontWeight(.bold)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(Theme.surfaceAlt)
                        .clipShape(Capsule())
                        .foregroundStyle(Theme.text)
                }

                Divider()
                    .overlay(Theme.border)

                // Intent section
                if let intent = resolution.intent {
                    Text("Intent")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text("Action:")
                                .foregroundStyle(Theme.textMuted)
                            Text(intent.actionType)
                                .foregroundStyle(Theme.text)
                        }
                        if let skill = intent.skill {
                            HStack {
                                Text("Skill:")
                                    .foregroundStyle(Theme.textMuted)
                                Text(skill)
                                    .foregroundStyle(Theme.text)
                            }
                        }
                        HStack {
                            Text("DC:")
                                .foregroundStyle(Theme.textMuted)
                            Text("\(intent.difficultyClass)")
                                .fontWeight(.bold)
                                .foregroundStyle(Theme.text)
                            Text("Modifier:")
                                .foregroundStyle(Theme.textMuted)
                            Text("\(intent.modifier)")
                                .fontWeight(.bold)
                                .foregroundStyle(Theme.text)
                        }
                    }
                    .font(.subheadline)

                    Divider()
                        .overlay(Theme.border)
                }

                // Check result section
                if let check = resolution.checkResult {
                    Text("Check Result")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    HStack(spacing: 16) {
                        VStack {
                            Text("Roll")
                                .font(.caption)
                                .foregroundStyle(Theme.textMuted)
                            Text("\(check.roll)")
                                .font(.title3)
                                .fontWeight(.bold)
                                .foregroundStyle(Theme.text)
                        }
                        VStack {
                            Text("Total")
                                .font(.caption)
                                .foregroundStyle(Theme.textMuted)
                            Text("\(check.total)")
                                .font(.title3)
                                .fontWeight(.bold)
                                .foregroundStyle(Theme.text)
                        }
                        VStack {
                            Text("Outcome")
                                .font(.caption)
                                .foregroundStyle(Theme.textMuted)
                            Text(check.outcome)
                                .font(.subheadline)
                                .fontWeight(.bold)
                                .foregroundStyle(Theme.accent)
                        }
                    }

                    Divider()
                        .overlay(Theme.border)
                }

                // Effects section
                if !resolution.effects.isEmpty {
                    Text("Effects")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    ForEach(resolution.effects, id: \.targetId) { effect in
                        HStack {
                            Text(effect.effectType)
                                .font(.subheadline)
                                .foregroundStyle(Theme.text)
                            Spacer()
                            Text("magnitude: \(effect.magnitude)")
                                .font(.caption)
                                .foregroundStyle(Theme.textMuted)
                        }
                    }

                    Divider()
                        .overlay(Theme.border)
                }

                // Phase-aware button
                if resolution.phase == "IntentDeclared" {
                    Button("Resolve Check") {
                        Task { await viewModel.resolveCheck() }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(Theme.accent)
                }

                // Version
                HStack {
                    Spacer()
                    Text("Version \(resolution.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
