import SwiftUI

/// Styled button for presenting a narrative choice.
struct ChoiceButtonView: View {
    var index: Int = 0
    let label: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                if index > 0 {
                    Text("\(index)")
                        .font(.headline)
                        .foregroundStyle(Theme.accent)
                        .frame(width: 28)
                }
                Text(label)
                    .foregroundStyle(Theme.text)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding()
            .background(Theme.surfaceAlt)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Theme.border, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
        .buttonStyle(.plain)
    }
}
