import SwiftUI

/// A single row in the inventory list.
struct InventoryRowView: View {
    let inventory: InventorySummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(inventory.inventoryId.uuidString.prefix(8))
                .font(.headline)
                .foregroundStyle(Theme.accent)
            HStack {
                Label("\(inventory.itemCount) items", systemImage: "bag.fill")
                    .font(.subheadline)
                    .foregroundStyle(Theme.text)
                Spacer()
                Text("v\(inventory.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
