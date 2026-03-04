import SwiftUI

/// Settings screen for configuring the API base URL.
struct SettingsView: View {
    @Bindable var configuration: AppConfiguration

    var body: some View {
        NavigationStack {
            Form {
                Section("API Connection") {
                    TextField("Base URL", text: $configuration.baseURLString)
                        .textContentType(.URL)
                        .autocorrectionDisabled()
                        #if os(iOS)
                        .textInputAutocapitalization(.never)
                        .keyboardType(.URL)
                        #endif
                }

                Section {
                    LabeledContent("Resolved URL", value: configuration.baseURL.absoluteString)
                }
            }
            .scrollContentBackground(.hidden)
            .background(Theme.surface)
            .navigationTitle("Settings")
        }
    }
}
