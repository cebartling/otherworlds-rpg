import SwiftUI

/// Screen showing all characters.
struct CharacterListView: View {
    @State private var viewModel: CharacterListViewModel

    private let endpoint: CharacterEndpoint

    init(client: HTTPClientProtocol) {
        let ep = CharacterEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: CharacterListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.characters.isEmpty {
                    LoadingView(message: "Loading characters...")
                } else if viewModel.characters.isEmpty {
                    ContentUnavailableView(
                        "No Characters",
                        systemImage: "person.slash",
                        description: Text("No characters found.")
                    )
                } else {
                    characterList
                }
            }
            .navigationTitle("Characters")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadCharacters()
            }
            .refreshable {
                await viewModel.loadCharacters()
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

    private var characterList: some View {
        List {
            ForEach(viewModel.characters) { character in
                NavigationLink(value: character.characterId) {
                    CharacterRowView(character: character)
                }
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        Task { await viewModel.archiveCharacter(id: character.characterId) }
                    } label: {
                        Label("Archive", systemImage: "archivebox")
                    }
                }
                .listRowBackground(Theme.surface)
            }
        }
        .scrollContentBackground(.hidden)
        .background(Theme.surface)
        .navigationDestination(for: UUID.self) { characterId in
            CharacterDetailView(characterId: characterId, endpoint: endpoint)
        }
    }
}
