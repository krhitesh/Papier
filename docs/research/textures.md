# Texture catalog — the science behind each grain

Every Papier texture is procedural grain (`paper-grain`), distinguished by three
parameters: **`base_frequency`** (grain fineness, cycles across the 200px tile),
**`aspect`** (vertical:horizontal frequency ratio → directional *weave*), and
**`persistence`** (fBm amplitude falloff → surface *roughness*). This document
explains *why* each texture's parameters are set the way they are.

## Mechanisms we're designing against

1. **Specular-glare diffusion.** A matte micro-structure scatters specular
   (mirror-like) highlights into diffuse light, lowering peak glare luminance.
   Anti-glare surfaces measurably improved legibility and reduced visual fatigue
   vs. glossy (Lin et al., *Displays* 29, 2008), and specular glare is the most
   disturbing kind (Schenkman et al., *Displays* 20, 1999). **Evidence: strong.**
2. **Spatial-frequency separation (legibility).** Human contrast sensitivity
   peaks around 2–5 cycles/degree, and text legibility lives in those low/mid
   frequencies. Grain placed at **high spatial frequency** (small features) sits
   above that band, so it diffuses glare with minimal masking of letterforms.
   This is why every texture is kept *fine*. **Evidence: established vision
   science (contrast-sensitivity function / visual masking); not measured in the
   Paperman sources — principle-based.**
3. **Restraint ("more matte is not better").** Lin et al. found legibility/fatigue
   benefit was statistically flat across a **6× haze range**, and present only
   under bright light (1500–3000 lux, not 200 lux). So stronger texture buys
   little and risks legibility — the catalog spans *gentle → rough* rather than
   pushing everything to maximum. **Evidence: strong (Lin et al.).**
4. **Multi-orientation scatter (weave).** A directional micro-structure scatters
   specular highlights across orientations; a cross-weave broadens the angles of
   diffusion. **Evidence: physical-optics reasoning; not separately measured —
   principle-based.**

> Invariant across all textures: the grain is **neutral luminance only**
> (R=G=B, no hue shift) and **fine** (a regression test asserts no large blobs
> for every catalog entry). Color/warmth is a separate, opt-in control.

## The catalog

| Texture | base_freq (≈feature) | aspect | octaves | persistence | Reason (mechanism → who it's for) |
|---|---|---|---|---|---|
| **Classic Matte** | 48 (~4px) | 1.0 iso | 3 | 0.50 | Balanced fine isotropic micro-roughness. High-frequency grain diffuses glare (mech. 1) while staying above text frequencies (mech. 2). The safe default. |
| **Whisper Weave** | 44 (~4.5px) | 2.0 | 3 | **0.42** (smoothest) | Faint directional weave at the **lowest amplitude** → the "lowest effective treatment" (mech. 3) for text-heavy / bright-screen reading; minimal contrast cost. |
| **Sunbaked Parchment** | 34 (~6px) | 1.2 | **4** | **0.62** | Heavier multi-octave diffuse fiber for long / low-light sessions — more scattering surface (mech. 1) while the base stays fine enough to protect legibility (mech. 2). |
| **Saddle Linen** *(new)* | 32 | **3.5** (woven) | 3 | 0.50 | Pronounced linen cross-weave → multi-orientation glare scatter (mech. 4); aimed at **bright, multi-source lighting**, where Lin et al. saw the largest benefit. |
| **Painter's Press** *(new)* | 36 | 1.0 | **5** (roughest) | **0.68** (roughest) | Cold-press watercolor tooth: most high-frequency energy = **strongest diffuse scattering** (mech. 1) for the brightest environments. Base kept fine so the added energy is high-frequency, not legibility-masking blobs (mech. 2). |
| **Vellum Mist** *(new)* | **64** (~3px, finest) | 1.0 | 3 | **0.38** (lowest) | Sheer, near-translucent surface — the **legibility extreme** (mech. 3): highest spatial frequency = least overlap with text frequencies (mech. 2). Best for dim rooms / dense text. |

## How to read it as a spectrum

- **Legibility-first ↔ glare-first:** Vellum Mist → Whisper Weave → Classic Matte
  → Saddle Linen → Sunbaked Parchment → Painter's Press (sheerest/finest to
  roughest/most-diffusing). Pick toward the rough end only in bright rooms, where
  the evidence for benefit is strongest and the legibility cost is outweighed.
- **Isotropic vs woven:** Whisper Weave (subtle) and Saddle Linen (pronounced)
  add directional scatter; the rest are isotropic.

## Honesty caveats

- The cited studies used a *physical* matte film diffusing *real* reflected glare
  on *reflective* e-paper. Papier is a *digital* veil over an *emissive* display
  with no real specular glare to scatter — it approximates the matte *appearance*
  and contrast attenuation. So mechanisms 1 and 4 are weaker in the digital case;
  the firmly-supported levers are **restraint** (mech. 3) and **keeping grain
  fine** (mech. 2). The catalog is shaped accordingly (fine, restrained, with the
  default at the gentle end).
- No source gives an exact grain-frequency number for a digital overlay; the
  specific values are principle-based engineering choices within the safe band.

See also [`legibility-tuning.md`](legibility-tuning.md) for the veil/intensity
defaults derived from the same literature.
