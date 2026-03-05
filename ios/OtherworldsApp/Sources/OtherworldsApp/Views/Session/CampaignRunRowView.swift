import SwiftUI

/// A single row in the campaign run list.
struct CampaignRunRowView: View {
    let campaignRun: CampaignRunSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(campaignRun.runId.uuidString.prefix(8))
                .font(.headline)
                .foregroundStyle(Theme.accent)
            if let campaignId = campaignRun.campaignId {
                Text("Campaign: \(campaignId.uuidString.prefix(8))")
                    .font(.caption)
                    .foregroundStyle(Theme.textMuted)
            }
            HStack {
                Label("\(campaignRun.checkpointCount) checkpoints", systemImage: "flag.fill")
                    .font(.subheadline)
                    .foregroundStyle(Theme.text)
                Spacer()
                Text("v\(campaignRun.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
