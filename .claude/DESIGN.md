---
name: Stark Agency System
colors:
  surface: '#f9f9fa'
  surface-dim: '#dadadb'
  surface-bright: '#f9f9fa'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f3f3f4'
  surface-container: '#eeeeef'
  surface-container-high: '#e8e8e9'
  surface-container-highest: '#e2e2e3'
  on-surface: '#1a1c1d'
  on-surface-variant: '#4c4546'
  inverse-surface: '#2f3132'
  inverse-on-surface: '#f0f1f2'
  outline: '#7e7576'
  outline-variant: '#cfc4c5'
  surface-tint: '#5e5e5e'
  primary: '#000000'
  on-primary: '#ffffff'
  primary-container: '#1b1b1b'
  on-primary-container: '#848484'
  inverse-primary: '#c6c6c6'
  secondary: '#5d5f5f'
  on-secondary: '#ffffff'
  secondary-container: '#dfe0e0'
  on-secondary-container: '#616363'
  tertiary: '#000000'
  on-tertiary: '#ffffff'
  tertiary-container: '#1b1b1b'
  on-tertiary-container: '#848484'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#e2e2e2'
  primary-fixed-dim: '#c6c6c6'
  on-primary-fixed: '#1b1b1b'
  on-primary-fixed-variant: '#474747'
  secondary-fixed: '#e2e2e2'
  secondary-fixed-dim: '#c6c6c7'
  on-secondary-fixed: '#1a1c1c'
  on-secondary-fixed-variant: '#454747'
  tertiary-fixed: '#e2e2e2'
  tertiary-fixed-dim: '#c6c6c6'
  on-tertiary-fixed: '#1b1b1b'
  on-tertiary-fixed-variant: '#474747'
  background: '#f9f9fa'
  on-background: '#1a1c1d'
  surface-variant: '#e2e2e3'
  accent-green: '#00FF41'
  border-main: '#000000'
  surface-muted: '#F4F4F5'
typography:
  display-xl:
    fontFamily: Inter
    fontSize: 120px
    fontWeight: '800'
    lineHeight: 110px
    letterSpacing: -0.04em
  headline-lg:
    fontFamily: Inter
    fontSize: 64px
    fontWeight: '800'
    lineHeight: 72px
    letterSpacing: -0.03em
  headline-lg-mobile:
    fontFamily: Inter
    fontSize: 40px
    fontWeight: '800'
    lineHeight: 44px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Inter
    fontSize: 32px
    fontWeight: '700'
    lineHeight: 40px
    letterSpacing: -0.01em
  body-lg:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '400'
    lineHeight: 28px
  body-md:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  label-caps:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '700'
    lineHeight: 16px
    letterSpacing: 0.1em
spacing:
  base: 8px
  gutter: 24px
  margin-desktop: 64px
  margin-mobile: 20px
  border-width-thick: 3px
  border-width-thin: 1px
---

## Brand & Style

The design system is rooted in **Neo-Brutalist Minimalism**. It targets a high-end demographic of tech founders and creative leaders who value clarity, directness, and architectural precision. The aesthetic is unapologetically bold, favoring raw structural elements over decorative flourishes. 

The emotional response should be one of "Confidence through Constraint." By using a restricted palette and heavy-weight typography, the UI signals authority and modern technical proficiency. Key characteristics include high-contrast boundaries, oversized headers, and a total absence of traditional gradients or soft blurs.

## Colors

The palette is dominated by a binary relationship between **Absolute Black** and **Stark White**. This high-contrast foundation ensures maximum legibility and a striking visual impact. 

- **Primary & Secondary:** Used to create a reversible UI where "Dark Mode" is simply the inversion of the layout.
- **Neutral:** Used sparingly for background fills in complex data views or secondary containers to prevent visual fatigue.
- **Accent Green:** A high-vibrancy "Electric Green" is reserved exclusively for primary calls to action, success states, and critical brand moments. It acts as a digital highlighter against the monochrome canvas.

## Typography

This design system utilizes **Inter** exclusively to maintain a systematic, utilitarian feel. The hierarchy is driven by extreme scale rather than font variety.

- **Display & Headlines:** Set in Extra Bold (800) with tight tracking. Headlines should feel "heavy" and occupy significant screen real estate.
- **Body Text:** Standard weights (400) with generous line-height to maintain readability against the high-contrast background.
- **Labels:** Small, uppercase, and bold. Used for metadata and overlines to provide a technical, structured feel.

## Layout & Spacing

The layout follows a **Rigid Grid** philosophy. Elements are locked to a 12-column grid on desktop and a 4-column grid on mobile. 

- **Gutters:** 24px wide, often visualized through vertical borders or hard dividers.
- **Margins:** Large outer margins (64px+) are used to "frame" the content, making the software feel like a curated gallery or an architectural blueprint.
- **Rhythm:** All spacing (padding, gaps) must be multiples of 8px. Use 3px borders for primary containers and 1px borders for internal dividers.

## Elevation & Depth

Depth is conveyed through **Hard Shadows** and **Offset Layers** rather than Z-axis blurs.

- **No Ambient Shadows:** Do not use soft, diffused shadows.
- **Offset Shadows:** To indicate a raised element (like a button or a card), use a solid black rectangle offset by 4px to 8px to the bottom-right.
- **Tonal Stacking:** Use the #F4F4F5 neutral color to indicate "inset" or "background" areas, while the #FFFFFF white surfaces represent the top-most interaction layer.

## Shapes

The shape language is strictly **Sharp (0px)**. 

Every UI component—buttons, input fields, cards, and images—must have square corners. This reinforces the "Brutalist" architectural theme. The only exception to this rule is for specific iconography or "status pips" which may be circular to provide a necessary visual break from the grid.

## Components

- **Buttons:** Primary buttons are solid black with white text. On hover, they shift to show an **Offset Hard Shadow** (8px offset) or swap colors to the Accent Green.
- **Input Fields:** Thick 2px black borders. Labels are placed inside the border at the top-left using the `label-caps` style.
- **Cards:** White background with a 2px black border. Content is padded by 32px. Cards do not use shadows unless they are interactive (hover state).
- **Chips/Badges:** Small rectangular boxes with a 1px border. No rounding.
- **Lists:** Separated by horizontal 1px lines that span the full width of the container. Use a "hover state" that fills the entire row with the Neutral color.
- **Checkboxes:** Square boxes with a thick black border. When checked, they are filled with the Accent Green and a black checkmark.