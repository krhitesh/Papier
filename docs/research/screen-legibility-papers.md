# Screen legibility — annotated bibliography

A literature survey for Papier, produced by a deep-research harness: 5 parallel
search angles → 22 sources fetched → 90 claims extracted → **25 claims put
through 3-vote adversarial verification** (a claim survives only if it is *not*
refuted by ≥2 of 3 independent skeptics). 23 confirmed, **2 killed**, then merged
to 11 findings. Confidence tags below reflect that vote plus sample/recency.

Grouped by the seven requested sub-areas. Each entry: **citation → finding →
strength of evidence → what it means for Papier.**

---

## 1. Luminance contrast & contrast polarity (dark- vs light-mode)

This is the strongest, most convergent area in the literature.

- **Buchner & Baumgartner (2007).** *Text–background polarity affects performance
  irrespective of ambient illumination and colour contrast.* **Ergonomics 50(7),
  1036–1063.** <https://www.tandfonline.com/doi/abs/10.1080/00140130701306413>
  — Proofreading is consistently better with **positive polarity (dark text on
  light)**, independent of ambient lighting (dark vs office) and chromaticity.
  **Evidence: high.**
- **Piepenbrock, Mayr, Mund & Buchner (2013).** *Positive display polarity is
  advantageous for both younger and older adults.* **Ergonomics 56(7).** PMID
  23654206. <https://pubmed.ncbi.nlm.nih.gov/23654206/> — Positive-polarity
  advantage in **both** a Landolt-C acuity task and proofreading, for **both age
  groups**; "strongly recommended independent of observer's age." **High.**
- **Piepenbrock, Mayr & Buchner (2014).** *Smaller pupil size and better
  proofreading performance with positive than with negative polarity displays.*
  **Ergonomics 57(11), 1670–1677.** PMID 25135324.
  <https://pubmed.ncbi.nlm.nih.gov/25135324/> — **Mechanism:** brighter
  (positive-polarity) displays **constrict the pupil → sharper retinal image →
  better fine-detail perception.** Pupils measurably smaller under positive
  polarity. **High.** *Key caveat: the effect is a luminance/contrast phenomenon,
  not polarity per se — it shrinks when overall luminance is equated.*
- **Dobres, Chahine & Reimer (2017).** *The effects of Brightness, Age, and
  Polarity on glance-like legibility.* **Applied Ergonomics 60, 68–73.** PMID
  28166901. <https://www.sciencedirect.com/science/article/abs/pii/S0003687016302459>
  — Polarity advantage confirmed for glance reading; **negative-polarity-in-dim
  light is the worst case**, while bright ambient light improves all conditions
  and nearly erases the polarity gap. **High.**

**→ Papier:** bias toward **high effective luminance/contrast** of the reading
surface; the matte veil matters most in **dim environments**, where the
negative-polarity penalty is largest. A digital veil that lowers luminance works
*against* this mechanism — keep intensity restrained (consistent with our
legibility-first defaults). Polarity should stay **user-configurable**, not
forced (see caveats: cataract/visual-search cases can favor dark mode).

## 2. Anti-glare / matte vs glossy & specular glare

- **Lin, Lin, Hwang, Jeng & Liao (2008).** *Effects of anti-glare surface
  treatment, ambient illumination and bending curvature on legibility and visual
  fatigue of electronic papers.* **Displays 29(1), 25–32.**
  <https://www.sciencedirect.com/science/article/abs/pii/S0141938207000510>
  — **The single most product-relevant paper.** Subjects "performed better on
  legibility and felt less visual fatigue with an anti-glare treatment." **Evidence:
  medium** (single study; reflective e-paper, not emissive LCD/OLED).
- Supporting (fetched, glare mechanism): glossy displays produced significantly
  worse visual performance over a 2-hour task than glare-free displays
  (<https://pubmed.ncbi.nlm.nih.gov/30805993/>); disability glare arises from
  **intraocular straylight producing a veiling luminance** that lowers retinal
  contrast (<https://pubmed.ncbi.nlm.nih.gov/22445628/>); AG coatings work by a
  micro-roughened surface that **diffuses specular reflection**
  (Radiant Vision Systems, industry — *blog, not peer-reviewed*).

**→ Papier:** a digital matte overlay emulating anti-glare diffusion is
**supported for reflective-style reading**, but on an emissive screen it can't
diffuse *real* glare and a matte coating trades ~10–15% peak brightness/sharpness;
**preserve effective contrast/luminance** so the veil doesn't offset its own
benefit. This is exactly why our textures are kept fine and the veil light.

## 3. Spatial frequency, character/font size & the CSF

- **Legge & Bigelow (2011).** *Does print size matter for reading? A review of
  findings from vision science and typography.* **Journal of Vision 11(5):8.**
  <https://jov.arvojournals.org/article.aspx?articleid=2191906> — Seminal review:
  a **"fluent range"** of print sizes spans ~10× in angular x-height (**~0.2°–2°;
  ≈1.4 mm/4 pt to 14 mm/40 pt at 40 cm**) over which reading is at max speed; the
  **critical print size** is **≥ ~2× the acuity limit** for normal vision (more
  for low vision). **High.**
- **Bigelow (2019).** *Typeface features and legibility research.* **Vision
  Research 165, 162–172.**
  <https://www.sciencedirect.com/science/article/pii/S0042698919301087> —
  Light-stroke fonts: their negative space **reduces crowding** but at small
  visual angles **harms legibility by removing the black mass needed for letter
  recognition at low spatial frequencies** (ties typography to the CSF). **High.**

**→ Papier:** don't size to the acuity limit; ensure type sits inside the fluent
range. Grain should be **high spatial frequency** (small features) so it doesn't
overlap the low/mid frequencies carrying letter identity — the core rationale in
`textures.md`.

## 4. Typography on screens (typeface, weight, spacing)

- **Sawyer, Dobres, Chahine & Reimer (2020).** *The great typography bake-off:
  comparing legibility at-a-glance.* **Ergonomics.** PMID 32089101.
  <https://www.tandfonline.com/doi/full/10.1080/00140139.2020.1714748> — Across 8
  sans-serifs, **typefaces with more open shapes/contours outperform closed ones**
  (Frutiger best ~105 ms; Eurostile worst ~123 ms). Typeface is a functional
  ergonomic variable, not just aesthetic. **High.**
- **Dobres, Wolfe, Chahine & Reimer (2018).** *The effects of visual crowding,
  text size, and positional uncertainty on text legibility at a glance.* **Applied
  Ergonomics 70.** PMID 29866314.
  <https://applylab.org/assets/pdf/Dobres_AppErgo_2018.pdf> — Larger text and
  wider **leading** each improve legibility, but **wider leading does NOT fully
  compensate** for small size. **Crowded list/column displays cost much more
  processing time** than isolated words (t(29)=10.05, p<0.001). **High.**
- **Dobres, Reimer & Chahine (2016).** *The Effect of Font Weight and Rendering
  System on Glance-Based Text Legibility.* **AutomotiveUI '16 (ACM).**
  <https://dl.acm.org/doi/10.1145/3003715.3005454> — **No main effect** of weight
  or rendering, but a strong **weight × rendering interaction**: under good
  sub-pixel rendering lighter weights are *more* legible; under poor rendering the
  lightest weight degrades badly. **High.**

**→ Papier:** any overlay that **degrades effective rendering quality
disproportionately penalizes light-weight type** — preserve rendering fidelity
(our veil is low-opacity and doesn't blur). If a future heavier effect is added,
bias toward heavier weights / larger sizes.

## 5. Display technology (e-paper vs LCD/OLED, flicker)

- **Benedetto, Drai-Zerbib, Pedrotti, Tissier & Baccino (2013).** *E-readers and
  visual fatigue.* **PLOS ONE 8(12): e83676.**
  <https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0083676> —
  **E-Ink produced visual-fatigue measures indistinguishable from paper; LCD
  (Kindle Fire HD) was significantly more fatiguing** than both. **Evidence:
  medium** (n=12, 2013 hardware; authors flag device-dependence).

**→ Papier:** emulating paper-like / reflective, **diffuse, low-flicker**
qualities plausibly reduces fatigue vs raw emissive LCD — but this is the weakest-
sampled area; don't overclaim.

## 6. Visual fatigue / computer vision syndrome / blink rate

- **Sheppard & Wolffsohn (2018).** *Digital eye strain: prevalence, measurement
  and amelioration.* **BMJ Open Ophthalmology 3(1): e000146.**
  <https://bmjophth.bmj.com/content/3/1/e000146> — Definitive modern review; DES
  affects ≥50% of users and splits into accommodative/binocular vs ocular-surface
  (dry-eye/blink) mechanisms. *(Fetched; no standalone claim isolated to a
  Papier-specific lever survived verification.)*
- Blink-rate / incomplete-blinks ↔ CVS: *Optom Vis Sci* (2013)
  <https://journals.lww.com/optvissci/Abstract/2013/05000/Blink_Rate,_Incomplete_Blinks_and_Computer_Vision.11.aspx>.

**→ Papier (honest):** blink rate appears only *indirectly* (as a fatigue measure
in Benedetto 2013). **No verified standalone claim** tied reduced blink rate /
CVS specifically to a matte overlay at equal luminance. Treat blink/fatigue
benefit as plausible-but-unproven for a digital veil. *(Open question below.)*

## 7. Blue light & color temperature — **evidence is weak**

- **Singh, Keller, Busija, et al. (2023).** *Blue-light filtering spectacle lenses
  for visual performance, sleep, and macular health in adults.* **Cochrane
  Database of Systematic Reviews, CD013244.**
  <https://www.cochranelibrary.com/cdsr/doi/10.1002/14651858.CD013244.pub2/full>
  — Systematic review: **no clear benefit** of blue-light-filtering lenses for eye
  strain/visual performance.
- **Chang, Aeschbach, Duffy & Czeisler (2015).** *Evening use of light-emitting
  eReaders…* **PNAS 112(4).** <https://www.pnas.org/doi/10.1073/pnas.1418490112>
  — Evening LE-eReader use affects *circadian/sleep* — not a *legibility* or
  *eye-strain-while-reading* result.

**→ Papier:** **NO blue-light / color-temperature finding was confirmed.** Treat
warmth/color-temperature as a **comfort/preference** feature, **not** an
evidence-backed legibility or eye-strain intervention. (Consistent with our
warmth default = 0 / neutral.)

---

## Refuted in verification (did NOT survive the skeptics)

- ❌ *"LCD and e-Ink produce very similar fatigue/strain outcomes"* — **killed
  0–3.** (Siegenthaler/Kang line, *Ophthalmic & Physiological Optics* 2012,
  <https://link.springer.com/content/pdf/10.1111/j.1475-1313.2012.00928.x.pdf>.)
  Contradicts Benedetto 2013; the "all displays equivalent" framing was rejected.
- ❌ *"Display technology per se isn't the determinant; image quality is the
  crucial factor"* — **killed 1–2.** Overstated from the same source.

## Framing caveats (apply to all of the above)

1. The polarity "advantage" is fundamentally a **luminance/contrast** effect
   (pupil constriction), not "light mode" per se — tune for **effective
   luminance/contrast**, and it's largest in dim light.
2. Much of the glance-legibility corpus (Dobres/Sawyer/Reimer, MIT AgeLab +
   Monotype) used **time-pressured glance/lexical-decision** tasks (automotive/
   notification UIs); generalization to long-form reading is reasonable but not
   identical.
3. Display-hardware findings rest on **small samples (n≈12), 2013-era devices**,
   explicitly device-dependent.
4. **Boundary condition:** readers with cloudy ocular media (cataracts) and some
   visual-search tasks can favor dark mode → keep polarity user-configurable.

## Open questions (unresolved by current evidence)

1. Does the polarity/luminance advantage from short tasks hold for **prolonged
   continuous reading**, where fatigue/adaptation differ?
2. Direct controlled evidence on **blink rate / tear-film / CVS attributable to a
   matte overlay** vs a glossy emissive screen *at equal luminance*? (None found.)
3. On **emissive LCD/OLED** (Papier's target), does digital matte emulation
   deliver the e-paper benefit, or does the unavoidable luminance reduction offset
   it given the luminance-driven mechanism?
4. Any **color-temperature/blue-light** setting with a *replicated* peer-reviewed
   reading/eye-strain effect distinguishable from placebo?

---

*Method: 104 agents, ~3.5M tokens. Confirmed findings are 3-0 unless noted; two
claims were actively refuted (above). Sources are linked inline; primary
peer-reviewed unless marked otherwise (one industry blog flagged).*
