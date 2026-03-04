import SwiftUI

/// Tab-based root navigation for the app.
struct AppTabView: View {
    let configuration: AppConfiguration

    var body: some View {
        TabView {
            Tab("Narratives", systemImage: "book.pages") {
                NarrativeSessionListView(client: configuration.makeHTTPClient())
            }

            Tab("Settings", systemImage: "gear") {
                SettingsView(configuration: configuration)
            }
        }
    }
}
