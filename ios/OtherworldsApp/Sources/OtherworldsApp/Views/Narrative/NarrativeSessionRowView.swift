import SwiftUI

/// A single row in the narrative session list.
struct NarrativeSessionRowView: View {
    let session: NarrativeSessionSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Session")
                .font(.headline)
            Text(session.sessionId.uuidString.prefix(8))
                .font(.caption)
                .foregroundStyle(.secondary)
            HStack {
                if let sceneId = session.currentSceneId {
                    Label(sceneId, systemImage: "book.pages")
                        .font(.subheadline)
                } else {
                    Text("No scene")
                        .font(.subheadline)
                        .foregroundStyle(.tertiary)
                }
                Spacer()
                Text("v\(session.version)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}
