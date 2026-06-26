# Legibility tuning research — anti-glare surface treatment

Evidence-backed guidance for Papier's overlay knobs, sourced from Paperman's
cited reference [1].

## Source

Po-Hung Lin, Yu-Ting Lin, Sheue-Ling Hwang, Shie-Chang Jeng, Chi-Chang Liao,
**"Effects of anti-glare surface treatment, ambient illumination and bending
curvature on legibility and visual fatigue of electronic papers,"** *Displays*
29 (2008) 25–32. (The cited "[1]" study; note it is *Displays* 2008, not
Applied Ergonomics 2009 — same author group / topic.)

Design: 45 subjects, letter-search task on 12-pt pseudo-text (19 lines × 71
chars), within-subject factor = surface treatment (3 levels), between-subject
factors = ambient illumination (200 / 1500 / 3000 lux) and bending curvature
(±10 cm, plane). Measures: search time, accuracy, change of CFF (objective
fatigue), subjective fatigue questionnaire (6 items, 10-pt scale).

---

## Findings (cited)

### F1 — Treatment levels tested and their measured optical properties (Table 1, p.2)

| Treatment | Haze % | Glare | Contrast ratio (CR) |
|-----------|--------|-------|---------------------|
| Transparency (no treatment) | – | **Very high** | 3.1 |
| AG30 (anti-glare) | 7.5 | High | 5.5 |
| AG150 (anti-glare) | 43 | **Extremely low** | 5.0 |

Samples were Nitto Denko AG layers laminated over a printed image (Sec. 2.1, p.2).

**Critical, non-obvious result:** the anti-glare matte layers *raised* measured
contrast ratio (3.1 → 5.0–5.5) rather than lowering it. In this reflective
e-paper setup the dominant legibility killer is *reflected glare washing out the
image*, so diffusing that glare nets a contrast **gain**. The matte's text-
blurring penalty did not show up as a net legibility loss at either haze level.

### F2 — Search time: anti-glare BEST, transparency WORST (Tables 2–4, Fig. 4; p.5, Sec. 3.1 / 4.1.1)

Mean search time (lower = faster = better legibility), Table 2:

- Transparency: **678.44 s** (worst)
- 7.5% haze: 656.67 s
- 43% haze: **654.71 s** (best)

Surface treatment was *the only significant factor* on search time
(F(2,72)=3.407, p<0.05). Paired tests (Table 4): 43%–transparency p=.028 and
7.5%–transparency p=.042 both significant; **43% vs 7.5% haze p=.852 — not
significant.** Illumination and curvature were not significant.

### F3 — Subjective visual fatigue: anti-glare BEST, transparency WORST (Tables 2,5–7, Figs.5–6; pp.4–7)

Mean subjective fatigue (lower = less fatigue), Table 2:

- Transparency: **20.29** (worst)
- 7.5% haze: 15.67
- 43% haze: **14.44** (best)

Surface treatment strongly significant (F(2,72)=31.486, p<0.01). Treatment ×
illumination interaction significant (p<0.05). Simple-effects:
- **200 lux: NOT significant** (F(2,24)=2.093, p>0.05) — at low light, matte vs.
  transparent indistinguishable.
- 1500 lux: significant (p<0.01); 3000 lux: significant (p<0.01).
- Paired tests at 1500 & 3000 lux: both haze levels beat transparency (p<0.01),
  but **43% vs 7.5% haze never significant** (p=.344 at 1500 lux, p=.145 at 3000).

### F4 — Accuracy and objective fatigue (CFF): no significant effect (Sec. 3.2, 3.3, 4.1.2, 4.2.2)

No independent variable significantly affected accuracy (subjects traded speed
for accuracy) or change of CFF. So the legibility benefit is a *speed/comfort*
benefit, not an error-rate or retinal-fatigue benefit.

### F5 — The glare/legibility tradeoff is NOT quantified as a matte-blur penalty

The paper does **not** give a haze threshold where text blur begins to hurt
reading. Within its range (up to 43% haze) more haze never hurt and trended
slightly better. The only "where it hurts" datum is on the *glare* side: Jeng et
al. [9] (cited p.2) found e-paper legibility rises with illumination 200→1500
lux then *falls above 1500 lux* due to reflected glare — which anti-glare
treatment mitigates. There is no upper haze bound in this study.

### F6 — Authors' endorsed/optimal level (Sec. 4.3, 5; pp.7–8)

Anti-glare treatment is "necessary to meet the ergonomic demand" and shows
utility under higher illumination. Because 43% and 7.5% haze were
statistically indistinguishable, the authors recommend the **cheaper / lower
treatment (7.5% haze)** — i.e. the *minimum* effective matte, not the maximum.

---

## Mapping to Papier's knobs

Important framing: the paper is about a *physical matte film over reflective
e-paper killing real reflected glare*. Papier is a *digital neutral veil + grain
alpha-composited over an emissive monitor*. There is no real glare to diffuse,
so Papier's veil cannot reproduce the paper's contrast **gain** — alpha-
compositing a light veil only *attenuates* contrast (the code comments
acknowledge this). The paper therefore supports Papier's *direction and intent*
(matte/anti-glare aids comfort and reading speed under bright light) but gives
**no direct numeric calibration** for an additive digital veil. Most knobs are
"silent" in a strict numeric sense; the honest mappings are principle-based.

### Tuning recommendations

| Knob (file) | Current | Paper says | Recommended action | Evidence |
|---|---|---|---|---|
| `VEIL_TONE` (grain.rs) | 0.78 | Silent on absolute veil lightness. Principle: anti-glare must not *cost* contrast; in the study it raised CR. A digital veil can only lower contrast. | **Keep, lean lighter.** Light veil (0.78) minimizes contrast loss vs. a grey matte — aligned with "don't sacrifice text contrast." No number to change it to. Avoid lowering it (greyer = more contrast loss = the one thing the paper's matte avoided). | F1 (CR 3.1→5.0–5.5); grain.rs comment |
| `GRAIN_AMP` (grain.rs) | 0.10 | Silent (no grain-amplitude analog; haze ≠ luminance-noise amplitude). Principle: matte tooth helped, more tooth (43% vs 7.5% haze) did **not** help further and didn't hurt. | **Keep.** "More matte ≠ better" supports a modest, non-aggressive amplitude. No basis to raise it. | F2, F3 (43% n.s. vs 7.5%) |
| `MIN_INTENSITY` / `MAX_INTENSITY` 0.15–0.30, default 0.18 (settings.rs) | 0.15–0.30 / 0.18 | Silent on absolute opacity. Principle: lowest effective treatment is preferred; benefit was equal across a 6× haze range. | **Keep range; keep default at the low end.** The "cheaper/lower is equal" finding argues for defaulting toward MIN, not MAX. Current 0.18 (near the floor) is well-placed; do **not** raise the default. | F6 (recommend cheaper 7.5%); F2/F3 |
| `warmth` 0..1, default 0.3 (settings.rs) | 0.3 | **Silent / arguably unsupported.** Paper's anti-glare matte was a neutral diffuser; the related Paperman grain spec is explicitly *saturate=0, no hue shift*. The study never adds tint and CFF/fatigue gains came from glare diffusion, not color. | **Flag:** warmth has no legibility/fatigue support in this source. A non-zero default (0.3) introduces a hue shift the paper's treatment did not have. Consider defaulting warmth to **0.0** (neutral) for the legibility-optimized baseline; keep the slider for aesthetic preference. | F1 (neutral matte); grain.rs/lib.rs "no hue" invariant |
| grain `base_frequency` 48/40/32 (≈4/5/6 px features) (paper-grain) | 48/40/32 | **Silent.** Paper specifies haze % (a scatter property), not a spatial grain frequency, and never relates feature size to reading. | **Keep — no evidence to change.** Fine grain (small features, well below the 12-pt glyph stroke) is the safe choice: it adds matte tooth without forming structure that competes with letterforms. The paper offers no number here. | (no datum; reasoned) |

---

## Most important takeaways

1. **Anti-glare helps legibility AND fatigue; transparency is worst.** Search
   time 678→655 s and subjective fatigue 20.3→14.4 going from no-treatment to
   matte (F2, F3). Papier's whole premise is supported.
2. **"More matte" is NOT better.** 43% vs 7.5% haze was statistically identical
   on every measure. So a *modest* veil/grain is as good as an aggressive one —
   keep `GRAIN_AMP`, `MAX_INTENSITY`, and the default intensity restrained; bias
   the default toward the low end (F2, F3, F6).
3. **The benefit only appears under bright light** (1500/3000 lux), not at 200
   lux (F3). Implication for Papier: the overlay's value is environment-
   dependent — a low default intensity that the user raises in bright rooms fits
   the evidence better than a high fixed value.
4. **The matte did not cost contrast here** — it *raised* CR by killing real
   glare (F1). Papier's additive veil cannot replicate that; it can only lower
   contrast. So keep `VEIL_TONE` light (0.78 is good) to minimize the contrast
   penalty, and do not chase a darker/greyer matte.
5. **No haze-blur legibility threshold exists in this paper** — it never found a
   point where matte starts hurting reading (F5). Don't invent one; Papier's
   restraint should come from the "more isn't better" finding, not a fabricated
   blur limit.

## Flag — current value the evidence questions

- **`warmth` default = 0.3** is the one current value with no support and mild
  counter-evidence: the studied anti-glare matte was neutral and Paperman's own
  grain spec mandates *no hue shift*. For a legibility-first baseline, warmth
  should default to **0.0**. (Range/slider can stay for taste.)
