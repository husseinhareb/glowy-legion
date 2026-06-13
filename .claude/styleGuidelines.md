## Brand & Style

The design system is rooted in **Neo-Brutalist Minimalism**. It targets a high-end demographic of tech founders and creative leaders who value clarity, directness, and architectural precision. The aesthetic is unapologetically bold, favoring raw structural elements over decorative flourishes. 

The emotional response should be one of "Confidence through Constraint." By using a restricted palette and heavy-weight typography, the UI signals authority and modern technical proficiency. Key characteristics include high-contrast boundaries, oversized headers, and a total absence of traditional gradients or soft blurs.

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

## Components

- **Buttons:** Primary buttons are solid black with white text. On hover, they shift to show an **Offset Hard Shadow** (8px offset) or swap colors to the Accent Green.
- **Input Fields:** Thick 2px black borders. Labels are placed inside the border at the top-left using the `label-caps` style.
- **Cards:** White background with a 2px black border. Content is padded by 32px. Cards do not use shadows unless they are interactive (hover state).
- **Chips/Badges:** Small rectangular boxes with a 1px border. No rounding.
- **Lists:** Separated by horizontal 1px lines that span the full width of the container. Use a "hover state" that fills the entire row with the Neutral color.
- **Checkboxes:** Square boxes with a thick black border. When checked, they are filled with the Accent Green and a black checkmark.