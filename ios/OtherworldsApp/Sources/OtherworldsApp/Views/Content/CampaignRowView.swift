import SwiftUI

/// A single row in the campaign list.
struct CampaignRowView: View {
    let campaign: CampaignSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(campaign.campaignId.uuidString.prefix(8))
                .font(.headline)
                .foregroundStyle(Theme.accent)
            HStack {
                Text(campaign.phase)
                    .font(.caption)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Theme.surfaceAlt)
                    .clipShape(Capsule())
                    .foregroundStyle(Theme.text)
                if let hash = campaign.versionHash {
                    Text(hash.prefix(8))
                        .font(.caption)
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Text("v\(campaign.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
