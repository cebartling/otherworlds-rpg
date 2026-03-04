import SwiftUI

/// Screen showing all narrative sessions.
struct NarrativeSessionListView: View {
    @State private var viewModel: NarrativeSessionListViewModel

    private let endpoint: NarrativeEndpoint

    init(client: HTTPClientProtocol) {
        let ep = NarrativeEndpoint(client: client)
        self.endpoint = ep
        self._viewModel = State(initialValue: NarrativeSessionListViewModel(endpoint: ep))
    }

    var body: some View {
        NavigationStack {
            Group {
                if viewModel.isLoading && viewModel.sessions.isEmpty {
                    LoadingView(message: "Loading sessions...")
                } else if viewModel.sessions.isEmpty {
                    ContentUnavailableView(
                        "No Sessions",
                        systemImage: "book.closed",
                        description: Text("No narrative sessions found.")
                    )
                } else {
                    sessionList
                }
            }
            .navigationTitle("Narratives")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    if viewModel.isLoading {
                        ProgressView()
                    }
                }
            }
            .task {
                await viewModel.loadSessions()
            }
            .refreshable {
                await viewModel.loadSessions()
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

    private var sessionList: some View {
        List {
            ForEach(viewModel.sessions) { session in
                NavigationLink(value: session.sessionId) {
                    NarrativeSessionRowView(session: session)
                }
            }
        }
        .navigationDestination(for: UUID.self) { sessionId in
            NarrativeSessionDetailView(sessionId: sessionId, client: endpoint)
        }
    }
}
