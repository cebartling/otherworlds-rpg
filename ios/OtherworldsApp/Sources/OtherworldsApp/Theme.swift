import SwiftUI

/// Dark fantasy color theme matching the web app's aesthetic.
enum Theme {
    // MARK: - Surfaces

    /// Main background — #1a1d23
    static let surface = Color(red: 0.102, green: 0.114, blue: 0.137)

    /// Cards, headers — #252830
    static let surfaceAlt = Color(red: 0.145, green: 0.157, blue: 0.188)

    /// Interactive hover/pressed state — #2e3139
    static let surfaceHover = Color(red: 0.180, green: 0.192, blue: 0.224)

    // MARK: - Text

    /// Primary text (warm off-white) — #e0ddd5
    static let text = Color(red: 0.878, green: 0.867, blue: 0.835)

    /// Secondary/label text — #8a8780
    static let textMuted = Color(red: 0.541, green: 0.529, blue: 0.502)

    // MARK: - Accent

    /// Gold highlights, headings, buttons — #c9a84c
    static let accent = Color(red: 0.788, green: 0.659, blue: 0.298)

    /// Accent hover/pressed state — #d4b95e
    static let accentHover = Color(red: 0.831, green: 0.725, blue: 0.369)

    // MARK: - Border

    /// Dividers, card borders — #3a3d45
    static let border = Color(red: 0.227, green: 0.239, blue: 0.271)
}
