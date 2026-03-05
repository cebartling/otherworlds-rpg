import SwiftUI

/// A single row in the world snapshot list.
struct WorldSnapshotRowView: View {
    let worldSnapshot: WorldSnapshotSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(worldSnapshot.worldId.uuidString.prefix(8))
                .font(.headline)
                .foregroundStyle(Theme.accent)
            HStack {
                Label("\(worldSnapshot.factCount) facts", systemImage: "list.bullet")
                    .font(.subheadline)
                    .foregroundStyle(Theme.text)
                Label("\(worldSnapshot.flagCount) flags", systemImage: "flag.fill")
                    .font(.subheadline)
                    .foregroundStyle(Theme.text)
                Spacer()
                Text("v\(worldSnapshot.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
