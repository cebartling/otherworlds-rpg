import SwiftUI

/// Detail screen for a single world snapshot — shows facts, flags, dispositions, and commands.
struct WorldSnapshotDetailView: View {
    @State private var viewModel: WorldSnapshotDetailViewModel

    init(worldId: UUID, endpoint: WorldStateEndpoint) {
        self._viewModel = State(
            initialValue: WorldSnapshotDetailViewModel(
                worldId: worldId,
                endpoint: endpoint
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.worldSnapshot == nil {
                LoadingView(message: "Loading world snapshot...")
            } else if let snapshot = viewModel.worldSnapshot {
                worldSnapshotContent(snapshot)
            } else {
                ContentUnavailableView(
                    "World Snapshot Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("Could not load this world snapshot.")
                )
            }
        }
        .background(Theme.surface)
        .navigationTitle("World Snapshot")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await viewModel.loadWorldSnapshot()
        }
        .refreshable {
            await viewModel.loadWorldSnapshot()
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

    private func worldSnapshotContent(_ snapshot: WorldSnapshotDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text(snapshot.worldId.uuidString.prefix(8))
                    .font(.title2)
                    .foregroundStyle(Theme.accent)

                Divider()
                    .overlay(Theme.border)

                // Facts
                Text("Facts")
                    .font(.headline)
                    .foregroundStyle(Theme.accent)

                if snapshot.facts.isEmpty {
                    Text("No facts")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                } else {
                    ForEach(snapshot.facts, id: \.self) { fact in
                        Text(fact)
                            .font(.subheadline)
                            .foregroundStyle(Theme.text)
                    }
                }

                Button("Apply Effect") {
                    Task { await viewModel.applyEffect(factKey: "new_fact") }
                }
                .buttonStyle(.borderedProminent)
                .tint(Theme.accent)

                Divider()
                    .overlay(Theme.border)

                // Flags
                Text("Flags")
                    .font(.headline)
                    .foregroundStyle(Theme.accent)

                if snapshot.flags.isEmpty {
                    Text("No flags")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                } else {
                    ForEach(snapshot.flags.sorted(by: { $0.key < $1.key }), id: \.key) { key, value in
                        HStack {
                            Text(key)
                                .font(.subheadline)
                                .foregroundStyle(Theme.text)
                            Spacer()
                            Text(value ? "true" : "false")
                                .font(.subheadline)
                                .fontWeight(.bold)
                                .foregroundStyle(value ? .green : Theme.textMuted)
                        }
                    }
                }

                Button("Set Flag") {
                    Task { await viewModel.setFlag(flagKey: "new_flag", value: true) }
                }
                .buttonStyle(.borderedProminent)
                .tint(Theme.accent)

                Divider()
                    .overlay(Theme.border)

                // Dispositions
                Text("Dispositions")
                    .font(.headline)
                    .foregroundStyle(Theme.accent)

                if snapshot.dispositionEntityIds.isEmpty {
                    Text("No dispositions")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                } else {
                    ForEach(snapshot.dispositionEntityIds, id: \.self) { entityId in
                        Text(entityId.uuidString.prefix(8))
                            .font(.subheadline)
                            .foregroundStyle(Theme.text)
                    }
                }

                // Version
                HStack {
                    Spacer()
                    Text("Version \(snapshot.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
