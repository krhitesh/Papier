# Research → product feature map

Translating the **verified** screen-legibility findings
([`screen-legibility-papers.md`](screen-legibility-papers.md): 25 claims through
3-vote adversarial verification, 23 confirmed / 2 refuted) into concrete Papier
product decisions.

**Framing constraint:** Papier is an **overlay** — it composites a matte veil
over arbitrary apps. It cannot change another app's typeface, size, leading, or
light/dark polarity. So findings split into: (A) what Papier already embodies,
(B) what justifies new features, (C) what is out of scope or not evidence-backed.

## A. Already embodied (research validates current design)

| Verified finding | Papier feature | Status |
|---|---|---|
| Anti-glare/matte improves legibility & reduces fatigue (Lin 2008) | matte veil + 6 textures | ✓ core premise validated |
| E-Ink/paper (diffuse, low-flicker) less fatiguing than LCD (Benedetto 2013) | **static** grain, zero animation, diffuse texture | ✓ |
| Polarity advantage is a **luminance/contrast** effect — shrinks when luminance is cut (Piepenbrock 2014) | restrained defaults: intensity floor 0.15, light `VEIL_TONE` | ✓ (veil must not crush luminance) |
| Low-spatial-frequency letter mass matters; degraded rendering penalizes light fonts (Bigelow 2019; Dobres 2016) | low-opacity veil that **never blurs**; fine high-frequency grain off text frequencies | ✓ (a constraint we honor) |
| Blue-light filtering shows **no clear benefit** (Cochrane 2023) | warmth defaults to **0 / neutral**, framed as comfort only | ✓ (no health claim) |

## B. New features justified by the evidence (prioritized)

### B1. Ambient-adaptive intensity — *highest priority*
**Backed by:** the veil reduces luminance (works against the polarity/luminance
mechanism, Piepenbrock 2014); the dim-light penalty is largest while the
matte/glare-diffusion benefit is strongest under **bright** ambient light
(Buchner 2007; Dobres 2017; Lin 2008).
**Feature:** drive intensity from the macOS ambient-light sensor (or a schedule)
— gentler veil in dim rooms (preserve luminance), stronger in bright rooms
(diffuse glare). User sets min/max bounds; manual override always available.
**Fit:** reuses the existing settings model + 30s timer loop.

### B2. Per-app intensity profiles
**Backed by:** legibility/luminance trade-off matters most over text.
**Feature:** extend the existing per-app *exclusion* list into per-app
*intensity* (e.g., a lighter veil over editors/terminals/readers). Reuses the
`NSWorkspace` frontmost-app machinery already in `exclusion.rs`.

### B3. "Reading mode" / contrast-loss cap
**Backed by:** preserve effective contrast (Buchner 2007; Piepenbrock 2014).
**Feature:** a toggle that caps max intensity lower (e.g. ≤20%) and forces
neutral warmth for text-heavy sessions.

### B4. Environment-tied texture guidance
**Backed by:** the bright-room benefit is where evidence is strongest.
**Feature:** surface the `textures.md` spectrum in the UI — "Vellum Mist
(dim / text-heavy) → Painter's Press (bright rooms)" — as guidance, not force.

## C. Out of scope or not supported (explicit non-goals)

- **Typeface openness, font size, leading, crowding** (Sawyer 2020; Legge &
  Bigelow 2011; Dobres 2018): real and important, but an overlay cannot set other
  apps' fonts/spacing. They belong in a reader/editor. Papier's only obligation
  is to **never degrade rendering** (already true). **Non-goal.**
- **Forcing positive polarity / light mode:** research says keep polarity
  **user-configurable** (cataract / visual-search users can prefer dark). Do not
  build a "force light mode." **Non-goal.**
- **Blue-light / color-temperature as eye-strain relief:** refuted/weak evidence
  (Cochrane 2023). Keep warmth strictly a comfort preference; **no health claim.**

## Recommended next step

Build **B1 (ambient-adaptive intensity)** first — it is the single feature most
directly supported by multiple confirmed claims and fits Papier's existing
architecture. Spec it via the brainstorming flow before implementing.

---
*Source of findings: [`screen-legibility-papers.md`](screen-legibility-papers.md).
Defaults rationale: [`legibility-tuning.md`](legibility-tuning.md). Texture
rationale: [`textures.md`](textures.md).*
