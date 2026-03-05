import SwiftUI

/// A single row in the character list.
struct CharacterRowView: View {
    let character: CharacterSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(character.name ?? "Unnamed")
                .font(.headline)
                .foregroundStyle(Theme.accent)
            Text(character.characterId.uuidString.prefix(8))
                .font(.caption)
                .foregroundStyle(Theme.textMuted)
            HStack {
                Label("\(character.experience) XP", systemImage: "star.fill")
                    .font(.subheadline)
                    .foregroundStyle(Theme.text)
                Spacer()
                Text("v\(character.version)")
                    .font(.caption2)
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(.vertical, 4)
    }
}
