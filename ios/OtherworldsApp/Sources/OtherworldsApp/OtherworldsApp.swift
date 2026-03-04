import SwiftUI

@main
struct OtherworldsApp: App {
    @State private var configuration = AppConfiguration()
    @State private var showSplash = true

    var body: some Scene {
        WindowGroup {
            if showSplash {
                SplashScreenView {
                    withAnimation(.easeInOut(duration: 0.5)) {
                        showSplash = false
                    }
                }
            } else {
                AppTabView(configuration: configuration)
            }
        }
    }
}
