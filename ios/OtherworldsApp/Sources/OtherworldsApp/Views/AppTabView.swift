import SwiftUI

/// Tab-based root navigation for the app.
struct AppTabView: View {
    let configuration: AppConfiguration

    var body: some View {
        TabView {
            Tab("Narratives", systemImage: "book.pages") {
                NarrativeSessionListView(client: configuration.makeHTTPClient())
            }

            Tab("Characters", systemImage: "person.3") {
                CharacterListView(client: configuration.makeHTTPClient())
            }

            Tab("Inventory", systemImage: "bag.fill") {
                InventoryListView(client: configuration.makeHTTPClient())
            }

            Tab("Sessions", systemImage: "play.rectangle.fill") {
                CampaignRunListView(client: configuration.makeHTTPClient())
            }

            Tab("World", systemImage: "globe") {
                WorldSnapshotListView(client: configuration.makeHTTPClient())
            }

            Tab("Rules", systemImage: "dice.fill") {
                ResolutionListView(client: configuration.makeHTTPClient())
            }

            Tab("Content", systemImage: "doc.text.fill") {
                CampaignListView(client: configuration.makeHTTPClient())
            }

            Tab("Play", systemImage: "bolt.fill") {
                ResolveActionView(client: configuration.makeHTTPClient())
            }

            Tab("Settings", systemImage: "gear") {
                SettingsView(configuration: configuration)
            }
        }
        .tint(Theme.accent)
    }
}
