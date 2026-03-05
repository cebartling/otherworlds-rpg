import SwiftUI

/// Detail screen for a single character — shows attributes, experience, and commands.
struct CharacterDetailView: View {
    @State private var viewModel: CharacterDetailViewModel

    init(characterId: UUID, endpoint: CharacterEndpoint) {
        self._viewModel = State(
            initialValue: CharacterDetailViewModel(
                characterId: characterId,
                endpoint: endpoint
            )
        )
    }

    var body: some View {
        Group {
            if viewModel.isLoading && viewModel.character == nil {
                LoadingView(message: "Loading character...")
            } else if let character = viewModel.character {
                characterContent(character)
            } else {
                ContentUnavailableView(
                    "Character Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("Could not load this character.")
                )
            }
        }
        .background(Theme.surface)
        .navigationTitle(viewModel.character?.name ?? "Character")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task {
            await viewModel.loadCharacter()
        }
        .refreshable {
            await viewModel.loadCharacter()
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

    private func characterContent(_ character: CharacterDetail) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Name
                Text(character.name ?? "Unnamed")
                    .font(.title2)
                    .foregroundStyle(Theme.accent)

                Divider()
                    .overlay(Theme.border)

                // Experience
                HStack {
                    Label("\(character.experience) XP", systemImage: "star.fill")
                        .font(.headline)
                        .foregroundStyle(Theme.text)
                    Spacer()
                    Button("Award XP") {
                        Task { await viewModel.awardExperience(amount: 50) }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(Theme.accent)
                }

                Divider()
                    .overlay(Theme.border)

                // Attributes
                if character.attributes.isEmpty {
                    Text("No attributes")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                } else {
                    Text("Attributes")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)

                    ForEach(character.attributes.sorted(by: { $0.key < $1.key }), id: \.key) { key, value in
                        HStack {
                            Text(key.capitalized)
                                .font(.subheadline)
                                .foregroundStyle(Theme.text)
                            Spacer()
                            Text("\(value)")
                                .font(.subheadline)
                                .fontWeight(.bold)
                                .foregroundStyle(Theme.text)
                        }
                    }
                }

                // Version
                HStack {
                    Spacer()
                    Text("Version \(character.version)")
                        .font(.caption2)
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .padding()
        }
        .background(Theme.surface)
    }
}
