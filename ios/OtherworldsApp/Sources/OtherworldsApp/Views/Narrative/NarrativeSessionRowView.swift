import SwiftUI

/// A single row in the narrative session list.
struct NarrativeSessionRowView: View {
    let session: NarrativeSessionSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Session")
                .font(.headline)
                .foregroundStyle(Theme.accent)
            Text(session.sessionId.uuidString.prefix(8))
                .font(.caption)
                .foregroundStyle(Theme.textMuted)
            HStack {
                if let sceneId = session.currentSceneId {
                    Label(sceneId, systemImage: "book.pages")
                        .font(.subheadline)
                        .foregroundStyle(Theme.text)
                } else {
                    Text("No scene")
                        .font(.subheadline)
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Text("v\(session.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
