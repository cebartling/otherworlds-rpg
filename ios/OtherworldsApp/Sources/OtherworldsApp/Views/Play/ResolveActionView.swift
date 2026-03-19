import SwiftUI

/// Play tab — form for submitting a resolve-action request to the backend.
struct ResolveActionView: View {
    @State private var viewModel: ResolveActionViewModel

    @State private var sessionIdText = ""
    @State private var worldIdText = ""
    @State private var actionType = "skill_check"
    @State private var skill = ""
    @State private var difficultyClass: Int = 15
    @State private var modifier: Int = 0

    init(client: HTTPClientProtocol) {
        self._viewModel = State(
            initialValue: ResolveActionViewModel(endpoint: PlayEndpoint(client: client))
        )
    }

    private var sessionId: UUID? { UUID(uuidString: sessionIdText) }
    private var worldId: UUID? { UUID(uuidString: worldIdText) }
    private var canSubmit: Bool { sessionId != nil && worldId != nil && !actionType.isEmpty }

    var body: some View {
        NavigationStack {
            Form {
                Section("Identifiers") {
                    TextField("Session ID (UUID)", text: $sessionIdText)
                        .autocorrectionDisabled()
                        #if os(iOS)
                        .textInputAutocapitalization(.never)
                        #endif
                    TextField("World ID (UUID)", text: $worldIdText)
                        .autocorrectionDisabled()
                        #if os(iOS)
                        .textInputAutocapitalization(.never)
                        #endif
                }

                Section("Action") {
                    TextField("Action type (e.g. skill_check)", text: $actionType)
                        .autocorrectionDisabled()
                        #if os(iOS)
                        .textInputAutocapitalization(.never)
                        #endif
                    TextField("Skill (optional)", text: $skill)
                        .autocorrectionDisabled()
                        #if os(iOS)
                        .textInputAutocapitalization(.never)
                        #endif
                    Stepper("Difficulty class: \(difficultyClass)", value: $difficultyClass, in: 1...30)
                    Stepper("Modifier: \(modifier)", value: $modifier, in: -5...10)
                }

                Section {
                    Button {
                        Task { await submit() }
                    } label: {
                        if viewModel.isLoading {
                            ProgressView()
                                .frame(maxWidth: .infinity)
                        } else {
                            Text("Resolve Action")
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .disabled(!canSubmit || viewModel.isLoading)
                }

                if let result = viewModel.result {
                    Section("Result") {
                        resultCard(result)
                    }
                }
            }
            .navigationTitle("Play")
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

    private func submit() async {
        guard let sessionId, let worldId else { return }
        let request = ResolveActionRequest(
            sessionId: sessionId,
            worldId: worldId,
            actionType: actionType,
            skill: skill.isEmpty ? nil : skill,
            targetId: nil,
            difficultyClass: Int32(difficultyClass),
            modifier: Int32(modifier),
            effects: []
        )
        await viewModel.resolveAction(request: request)
    }

    private func resultCard(_ result: ResolveActionResponse) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Label(result.correlationId.uuidString, systemImage: "link")
                .font(.caption)
                .foregroundStyle(Theme.textMuted)
            Divider()
            phaseRow("Intent", count: result.intentEventIds.count)
            phaseRow("Check", count: result.checkEventIds.count)
            phaseRow("Effects", count: result.effectsEventIds.count)
            phaseRow("World State", count: result.worldStateEventIds.count)
            phaseRow("Narrative", count: result.narrativeEventIds.count)
        }
    }

    private func phaseRow(_ label: String, count: Int) -> some View {
        HStack {
            Text(label)
                .foregroundStyle(Theme.text)
            Spacer()
            Text("\(count) event\(count == 1 ? "" : "s")")
                .foregroundStyle(Theme.accent)
                .font(.subheadline)
        }
    }
}
