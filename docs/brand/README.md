# Continuum Brand Manual

> Version: 1.0 | Date: 2026-05-26

---

## Brand Overview

**Continuum** is a concise and reliable Agent runtime. Our brand embodies the principles of continuity, reliability, and developer-centric design.

### Brand Promise

Continuum transforms chaotic Agent execution into predictable, debuggable workflows. We give developers control and visibility.

### Brand Personality

| Trait | Description |
|-------|-------------|
| **Reliable** | Consistent, trustworthy, dependable |
| **Transparent** | Open, clear, honest |
| **Developer-First** | Practical, intuitive, respectful of time |
| **Performant** | Efficient, optimized, fast |

---

## Logo

### Primary Logo

The Continuum logo represents the infinite loop of Agent execution with a central core engine. The interlocking "C" shapes symbolize continuity and the six-layer architecture.

**File**: `docs/assets/logo.svg`

### Logo Elements

1. **Circle Background** - Represents completeness and security
2. **Interlocking C Shapes** - Represents continuous execution and the Continuum name
3. **Center Core** - The heart of the system, the reliable engine
4. **Six Dots** - The six-layer architecture (Layer 0-5)
5. **Circuit Lines** - Connection and data flow between layers

### Logo Variations

| Variant | Usage |
|---------|-------|
| **Full Color** | Primary usage, digital interfaces, documentation |
| **Monochrome (White)** | Dark backgrounds |
| **Monochrome (Dark)** | Light backgrounds, print |

### Logo Clear Space

Maintain minimum clear space equal to the height of the "C" shape around the logo.

### Minimum Size

- Digital: 32x32 pixels
- Print: 12mm

---

## Color System

### Primary Colors

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Deep Blue** | `#1a1a2e` | 26, 26, 46 | Primary background, text |
| **Navy** | `#16213e` | 22, 33, 62 | Secondary background |
| **Dark Blue** | `#0f3460` | 15, 52, 96 | Accent background |

### Accent Colors

| Name | Hex | RGB | Usage |
|------|-----|-----|-------|
| **Cyan** | `#00d9ff` | 0, 217, 255 | Primary accent, CTAs, links |
| **Teal** | `#00ff88` | 0, 255, 136 | Secondary accent, success states |

### Functional Colors

| Name | Hex | Usage |
|------|-----|-------|
| **Success** | `#00ff88` | Success messages, positive states |
| **Warning** | `#ffb800` | Warning messages, cautions |
| **Error** | `#ff4757` | Error messages, destructive actions |
| **Info** | `#00d9ff` | Information, tips |

### Color Usage Guidelines

```
┌─────────────────────────────────────────────────────────┐
│  Background: Deep Blue (#1a1a2e)                        │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Card/Panel: Navy (#16213e)                       │  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │  Text: White (#ffffff)                      │  │  │
│  │  │  Accent: Cyan (#00d9ff)                     │  │  │
│  │  │  Secondary Accent: Teal (#00ff88)           │  │  │
│  │  └─────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Typography

### Primary Font Family

**Font**: Inter (or system sans-serif fallback)

```css
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
```

### Type Scale

| Level | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| H1 | 48px | 700 | 1.1 | Page titles |
| H2 | 32px | 600 | 1.2 | Section headers |
| H3 | 24px | 600 | 1.3 | Sub-sections |
| H4 | 18px | 600 | 1.4 | Card headers |
| Body | 16px | 400 | 1.6 | Paragraphs, descriptions |
| Small | 14px | 400 | 1.5 | Captions, metadata |
| Code | 14px | 400 | 1.5 | Code blocks |

### Code Font

**Font**: JetBrains Mono (or Fira Code fallback)

```css
font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
```

---

## Mascot: Nova

### Overview

**Nova** is Continuum's friendly AI assistant mascot. Nova embodies the helpful, transparent, and reliable nature of the Continuum agent runtime.

**File**: `docs/assets/mascot.svg`

### Character Design

| Feature | Description | Symbolism |
|---------|-------------|-----------|
| **Round Head** | Smooth, friendly shape | Approachability |
| **Digital Eyes** | Glowing cyan eyes | AI/technology |
| **Antenna** | Single antenna with glow | Connectivity, receiving |
| **Chest Display** | Continuum logo mark | Brand identity |
| **Raised Arm** | Greeting gesture | Helpful, welcoming |
| **Floating Particles** | Data points around body | Continuous data flow |

### Personality

Nova is:
- Helpful and patient
- Technically competent but approachable
- Transparent and honest
- Always ready to assist

### Usage Guidelines

- Use in documentation, onboarding, and marketing materials
- Do not modify proportions or colors
- Maintain friendly expression
- Keep clear space around the character

---

## Writing Style

### Voice

- **Technical but accessible**: Explain complex concepts simply
- **Direct and concise**: Get to the point quickly
- **Confident but humble**: Show expertise without arrogance
- **Encouraging**: Help developers succeed

### Terminology

| Term | Usage |
|------|-------|
| **Agent** | Autonomous AI entity that executes tasks |
| **Runtime** | The execution environment |
| **Session** | A single interaction context |
| **Checkpoint** | A saved state for recovery |
| **Layer** | Architectural component (0-5) |

### Examples

**Good**: "Run your agent with confidence. Continuum tracks every step."

**Avoid**: "Our revolutionary AI-powered platform enables unprecedented agent capabilities."

---

## Visual Elements

### Borders and Radius

| Element | Radius | Border |
|---------|--------|--------|
| Cards | 8px | 1px solid rgba(0, 217, 255, 0.2) |
| Buttons | 6px | none |
| Inputs | 4px | 1px solid rgba(0, 217, 255, 0.3) |
| Modals | 12px | none |

### Shadows

```css
/* Subtle elevation */
box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);

/* Medium elevation */
box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);

/* High elevation */
box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
```

### Gradients

```css
/* Primary gradient */
background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);

/* Accent gradient */
background: linear-gradient(90deg, #00d9ff 0%, #00ff88 100%);
```

---

## Usage Examples

### Terminal/CLI Theme

```
╭─────────────────────────────────────────────────────────╮
│  Continuum v1.0.0                              [●●●○○] │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  > continuum run "analyze project"                      │
│                                                         │
│  ◆ Initializing session...                              │
│  │  Model: claude-sonnet-4-6                            │
│  │  Provider: anthropic                                 │
│  │  Budget: $5.00 limit                                 │
│  ◆                                                      │
│  ├─ Parsing project structure...                        │
│  │  └─ Found 47 files                                   │
│  ├─ Analyzing dependencies...                           │
│  │  └─ 23 direct, 156 transitive                        │
│  ├─ Generating report...                                │
│                                                          │
│  ✓ Complete (1274 tokens, $0.02)                        │
│                                                         │
╰─────────────────────────────────────────────────────────╯
```

### Documentation Header

```
┌─────────────────────────────────────────────────────────┐
│  [Continuum Logo]                                       │
│                                                         │
│  Continuum Documentation                                │
│  A concise and reliable Agent runtime                   │
│                                                         │
│  Quick Start | Architecture | API Reference | Examples │
└─────────────────────────────────────────────────────────┘
```

---

## Do's and Don'ts

### Do

- Use approved color palette consistently
- Maintain adequate contrast ratios (4.5:1 minimum)
- Keep the logo proportionate and legible
- Use Nova in welcoming, helpful contexts

### Don't

- Stretch or distort logo or mascot
- Use colors outside the approved palette
- Add effects or modifications to brand assets
- Use low-resolution versions

---

## Asset Files

| Asset | Location | Format |
|-------|----------|--------|
| Primary Logo | `docs/assets/logo.svg` | SVG |
| Mascot (Nova) | `docs/assets/mascot.svg` | SVG |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-05-26 | Initial brand guidelines |

---

*Continuum Brand Manual v1.0*
