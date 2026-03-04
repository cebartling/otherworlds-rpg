import SwiftUI

@main
struct OtherworldsApp: App {
    @State private var configuration = AppConfiguration()

    var body: some Scene {
        WindowGroup {
            AppTabView(configuration: configuration)
        }
    }
}
