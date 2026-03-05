# ADR-0020: Cross-Platform Dark Fantasy Theme System

## Status

Accepted

## Context

The Otherworlds RPG engine serves two client platforms — an iOS app built with SwiftUI and a web app built with SvelteKit and Tailwind CSS. Both clients need a consistent visual identity that reinforces the game's dark fantasy aesthetic. Without a shared color specification, the platforms would drift apart visually, and players switching between devices would experience a jarring inconsistency.

The game's genre (dark fantasy tabletop RPG) calls for a dark, muted palette with warm metallic accents — think aged parchment, tarnished gold, and shadowed stone.

## Decision

Both platforms implement an identical color palette using platform-native mechanisms. The palette is a single dark theme with no light mode variant.

### Shared Hex Values

| Token         | Hex       | Usage                              |
|---------------|-----------|-------------------------------------|
| surface       | `#1a1d23` | Primary background                  |
| surface-alt   | `#252830` | Card / elevated surface background  |
| surface-hover | `#2e3139` | Hover / pressed state background    |
| accent        | `#c9a84c` | Primary accent (gold)               |
| accent-hover  | `#d4b95e` | Accent hover state                  |
| text          | `#e0ddd5` | Primary text (warm off-white)       |
| text-muted    | `#8a8780` | Secondary / disabled text           |
| border        | `#3a3d45` | Dividers and borders                |

### iOS Implementation (`Theme.swift`)

A `Theme` enum with static `Color` properties, using RGB decimal values derived from the hex palette:

```swift
enum Theme {
    static let surface = Color(red: 0.102, green: 0.114, blue: 0.137)       // #1a1d23
    static let surfaceAlt = Color(red: 0.145, green: 0.157, blue: 0.188)    // #252830
    static let surfaceHover = Color(red: 0.180, green: 0.192, blue: 0.224)  // #2e3139
    static let text = Color(red: 0.878, green: 0.867, blue: 0.835)          // #e0ddd5
    static let textMuted = Color(red: 0.541, green: 0.529, blue: 0.502)     // #8a8780
    static let accent = Color(red: 0.788, green: 0.659, blue: 0.298)        // #c9a84c
    static let accentHover = Color(red: 0.831, green: 0.725, blue: 0.369)   // #d4b95e
    static let border = Color(red: 0.227, green: 0.239, blue: 0.271)        // #3a3d45
}
```

Usage: `Theme.surface`, `Theme.accent`, etc. The enum cannot be instantiated — it serves purely as a namespace.

### Web Implementation (`app.css`)

Tailwind CSS 4's `@theme` block defines CSS custom properties:

```css
@theme {
    --color-surface: #1a1d23;
    --color-surface-alt: #252830;
    --color-accent: #c9a84c;
    --color-text: #e0ddd5;
    --color-text-muted: #8a8780;
    --color-accent-hover: #d4b95e;
    --color-surface-hover: #2e3139;
    --color-border: #3a3d45;
}
```

Usage: Tailwind utility classes like `bg-surface`, `text-accent`, `border-border`.

### No Light Mode

The dark theme is the only theme. The game's aesthetic is inherently dark — a light mode would undermine the visual identity. This simplifies both implementations by avoiding conditional theming logic.

## Consequences

### Positive

- Players see a consistent visual identity across iOS and web, reinforcing brand cohesion.
- Both platforms use their native theming mechanisms (Swift enums, CSS custom properties) — no cross-platform abstraction layer needed.
- The hex-value table serves as a single source of truth that both implementations derive from.
- Adding new tokens (e.g., `error`, `success`, `warning`) follows the same pattern on both platforms.

### Negative

- Color values must be manually synchronized — changing a hex value requires updating both `Theme.swift` and `app.css`.
- No light mode limits accessibility for users who prefer or need light backgrounds (future consideration).
- The iOS implementation uses RGB decimal approximations of the hex values, which could introduce sub-pixel color differences on some displays.

### Constraints

- All UI colors must come from the theme system. Do not use hardcoded hex values or system colors outside the theme.
- Any new color token must be added to both platforms simultaneously with identical hex values.
- The hex-value table in this ADR is the canonical reference — resolve discrepancies in favor of this table.
- Future per-campaign color schemes should extend (not replace) the base palette, adding campaign-specific accent colors while keeping the structural tokens (surface, text, border) unchanged.
