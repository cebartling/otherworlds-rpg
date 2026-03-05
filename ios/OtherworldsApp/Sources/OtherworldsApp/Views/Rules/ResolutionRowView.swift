import SwiftUI

/// A single row in the resolution list.
struct ResolutionRowView: View {
    let resolution: ResolutionSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(resolution.resolutionId.uuidString.prefix(8))
                .font(.headline)
                .foregroundStyle(Theme.accent)
            HStack {
                Text(resolution.phase)
                    .font(.caption)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Theme.surfaceAlt)
                    .clipShape(Capsule())
                    .foregroundStyle(Theme.text)
                if let outcome = resolution.outcome {
                    Text(outcome)
                        .font(.caption)
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Text("v\(resolution.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
