# Medusa Skill Framework — Technical Architecture

## Overview

Medusa is an ultra-fast skill scanner that uses **audit-based ranking** to objectively assess skill quality. It scans skill markdown files, extracts YAML frontmatter, measures content complexity, assigns a 9-tier ranking, and optionally maintains cross-session state through a dreaming process.

---

## Data Flow

```
SKILL.md files
    ↓
[WalkDir] Scan filesystem (max depth 4)
    ↓
[Rayon] Parallel frontmatter extraction (optional)
    ↓
[Regex] Parse YAML frontmatter
    ↓
[Audit] Measure 4 metrics: length, code blocks, steps, tech terms
    ↓
[Score] Calculate: 60% complexity + 30% value + 10% keyword bonus
    ↓
[Tier] Map 0–100 score → 10 tiers (Godlike → Poor)
    ↓
[Fusion] Detect similar skills via FxHash name + content hash
    ↓
[Dream] Record session snapshot, detect patterns (optional)
    ↓
[Outcomes] Assess against rubric criteria (optional)
    ↓
[Output] JSON / HTML / CSV / MD / SVG
```

---

## Module Map

| Module | File | Responsibility |
|--------|------|----------------|
| CLI | `src/main.rs` | Argument parsing, command dispatch, scoring algorithms, report generation |
| Dreaming | `src/dream.rs` | Session recording, pattern detection, consolidation, cross-session context, learning suggestions, dream diary |
| Outcomes | `src/outcomes.rs` | Rubric CRUD, weighted criterion assessment |
| Agents | `src/agents.rs` | Multi-agent orchestrated audit (4 specialized sub-audits) |
| Procedural | `src/procedural.rs` | Workflow detection and procedural memory |

---

## Scoring Algorithm

### Formula

```
experience = (complexity_score × 0.6) + (value_score × 0.3) + (keyword_bonus × 0.1)
```

Weights are configurable via `medusa.toml`.

### Component Breakdown

**Complexity Score** (60%):
- `content_length_score = min(content_length / 5000, 1.0) × 30`
- `code_blocks_score = min(code_blocks / 15, 1.0) × 30`
- `step_instructions_score = min(step_count / 8, 1.0) × 20`
- `tech_terms_score = min(term_count / 20, 1.0) × 20`
- Total: sum of above, capped at 100

**Value Score** (30%):
- Inverse gap penalty: `(1.0 - min(gap_count / 5, 1.0)) × 100`
- Bonuses for key areas: machine learning, ai, llm, data, python, cloud, infrastructure — each adds weighted points (max 100 with diminishing returns)

**Keyword Bonus** (10%):
- Matches skill name and gap text against high-value keywords
- Each match: `+2.0` points, capped at +20

### 10-Tier Mapping

| Tier | Range | Description |
|------|-------|-------------|
| Godlike | ≥95 | Exceptional mastery |
| Unique | ≥90 | Best-in-class |
| Legendary | ≥85 | Highly refined |
| Mythic | ≥80 | Strong |
| Epic | ≥75 | Solid |
| Ultra Rare | ≥65 | Competent |
| Rare | ≥55 | Developing |
| Uncommon | ≥45 | Basic |
| Common | ≥25 | Foundational |
| Poor | <25 | Minimal |

### Confidence

```
confidence = 100 - (gap_count × 10) - (5 if unfamiliar)
```

Confidence is how reliably the skill performs, not how good it is. Gaps and unfamiliarity reduce confidence.

---

## Dreaming Process

The dreaming process enables **cross-session learning** by recording snapshots of every scan and detecting patterns over time.

### Data Files

| File | Location | Content |
|------|----------|---------|
| `.medusa_dream.json` | Scan target directory (or parent walk-up) | Consolidated knowledge base with detected patterns, per-skill statistics |
| `.medusa_history.json` | Same as above | Raw session snapshots with timestamps |

### Commands

- `medusa dream <path>` — Manually trigger pattern detection (auto-runs after every scan)
- `medusa dream-status <path>` — Show full dream knowledge base report
- `medusa dream-reset <path>` — Clear all dream state and history
- `medusa dream-consolidate <path>` — Manually merge duplicate patterns

### Session Recording

Every scan, html, export, or audit command auto-records a snapshot via `dream::record_session`. Each snapshot captures:
- Skill scores and levels
- Gap list
- Timestamp

### Pattern Detection (`run_dream`)

Medusa compares current session data against history to detect:

| Pattern | Condition |
|---------|-----------|
| **Recurring Gap** | Same gap appears in ≥2 of last 5 sessions |
| **Improvement** | Score increased by >1.0 vs last session |
| **Decline** | Score dropped by >1.0 vs last session |
| **Stable Skill** | Score varies by <0.5 over last 3 sessions |
| **New Skill** | First appearance in history |
| **Resolved Gap** | Gap present in last session but not current |

### Memory Consolidation (`consolidate`)

After each dream cycle, consolidation runs automatically to:

1. **Merge duplicates** — Insights with same skill + type + gap text are merged (uses `max()` for occurrence counts to avoid inflation)
2. **Prune low-severity** — Stable insights with severity <3 are removed
3. **Capacity limit** — Max 200 insights; oldest pruned first

### Cross-Session Context

The `get_cross_session_summary` function returns human-readable insight lines for any skill:
- "Recurring gap: missing examples (seen in 3 of 5 sessions)"
- "Improved from 72.0 to 78.5"
- "Stable ~82.0 across 3 sessions"

These appear in audit reports, HTML reports, and JSON scan output.

---

## Outcomes Framework

The outcomes framework provides **weighted rubric-based assessment** for any skill.

### Data File

| File | Location | Content |
|------|----------|---------|
| `.medusa_outcomes.json` | Scan target directory | Rubric definitions per skill ID |

### Commands

- `medusa outcome-add <path> <skill_id>` — Add default rubric (4 criteria) for a skill
- `medusa outcome-list <path>` — List all defined rubrics
- `medusa outcome-remove <path> <skill_id>` — Remove a rubric

### Default Rubric

Four equally weighted criteria (25% each):

| Criterion | Weight | Scoring |
|-----------|--------|---------|
| Content Length | 0.25 | ≥5000 chars = 100 |
| Code Examples | 0.25 | ≥10 code blocks = 100 |
| Step Instructions | 0.25 | ≥5 step instructions = 100 |
| Technical Depth | 0.25 | ≥20 tech terms = 100 |

### Assessment

```
score = sum(criterion_weight × criterion_score for each criterion) × 100
```

Tiers:
- **Good** — score ≥ 80
- **Needs Improvement** — score ≥ 50
- **Poor** — score < 50

---

## Learning Paths

Learning paths are **built-in suggestions** mapped to detected gap patterns:

| Gap Pattern | Suggested Fix | Expected Impact |
|-------------|---------------|-----------------|
| No code examples | Add practical code examples | +15 experience |
| Missing key features | Describe core capabilities | +10 experience |
| No prerequisites | Add prerequisites section | +8 experience |
| Missing or short | Expand with detailed steps | +12 experience |
| No step instructions | Add ordered step instructions | +10 experience |
| Step instructions missing | Number step instructions | +10 experience |
| No technical terms | Add industry-standard terminology | +8 experience |
| Lacks examples | Include more examples | +10 experience |
| Missing configuration | Add configuration options | +8 experience |
| No troubleshooting | Add common issue solutions | +8 experience |
| Default | Review skill content | +5 experience |

The `medusa learning-path <path> <skill_id>` command shows the full learning plan for a skill, and inline suggestions appear under recurring gaps in dream reports.

---

## Multi-Agent Orchestration

The orchestrated audit (`medusa orchestrate`) decomposes a skill audit into 4 specialized sub-audits:

| Agent | Weight | Evaluates | Metrics |
|-------|--------|-----------|---------|
| Documentation Quality | 25% | Content length, step instructions, heading structure, readability | Line length, heading count, structure |
| Code Quality | 30% | Code blocks, language diversity, code-to-text ratio, code gaps | Block count, language variety, ratio |
| Dependency Health | 20% | Dependency count, fusion risk, tech term diversity, alignment | Dep count, fusion flags, term richness |
| Learning Value | 25% | Gap severity, complexity value, improvement history, confidence | Gap count, score depth, history length |

### Scoring

Each agent produces a 0–100 score with findings. The overall score is a weighted average:

```
overall = (doc_score × 0.25) + (code_score × 0.30) + (dep_score × 0.20) + (learn_score × 0.25)
```

Status tiers: Good (≥70), Needs Work (≥40), Poor (<40).

---

## Dream Diary

The dream diary (`medusa dream-diary`) produces a narrative timeline of skill evolution:

- **Session overview**: count, date range, average experience trend
- **Per-skill timeline**: experience over sessions with ASCII bar chart
- **Gap evolution**: shows which gaps appeared/resolved between sessions
- **Pattern summary**: recurring gaps, resolved gaps, improvements, declines, new skills

Supports `--output <file.md>` for Markdown export.

---

## Configurable Dreaming Parameters

Dreaming behavior is configured in `medusa.toml` under the `[dreaming]` section:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `frequency_scans` | 1 | Run dream every N scans |
| `retention_percent` | 0.8 | Memory retention (0.0–1.0, higher = keep more) |
| `auto_apply` | true | Auto-apply detected pattern changes |
| `max_insights` | 200 | Maximum insights after consolidation |

View current settings: `medusa dream-params <path>`

---

## Procedural Memory

Procedural memory (`medusa procedural-list`, `medusa procedural-show`) stores step-by-step workflows extracted from skill content.

### Workflow Detection

During each scan, Medusa scans skill content for step sequences (numbered lists like `1. ...` or bullet lists like `- ...`). Sequences with 3+ consecutive steps are saved as workflows with:

- Auto-categorized step types: Setup, Execution, Verification, Implementation, Learning, Documentation, General
- Source skill tracking
- Usage count and timestamps

### Data File

`.medusa_procedural.json` — auto-created during scan, stores all detected workflows.

---

## Cross-Agent Memory Sharing

Memory sharing (`medusa memory-export`, `medusa memory-import`) enables collaboration between Medusa instances.

### Export Bundle

Exports all three memory stores into a single JSON file:
- DreamKnowledgeBase (patterns, insights)
- ProceduralMemory (workflows)
- OutcomeStore (rubrics)

Includes source identifier and export timestamp.

### Import

Imports a bundle from another instance and merges with local data:
- **Dream insights**: new insights appended (deduplication by ID), tagged with source
- **Workflows**: new workflows appended (deduplication by name)
- **Rubrics**: new rubrics added (deduplication by skill ID)

Use `--source <name>` to tag imported data with a source identifier for provenance tracking.

---

## Scanning Modes

### Parallel (Default)

Uses Rayon's `par_bridge` for concurrent file processing. Measured 46% faster than sequential.

### Sequential

Pass `--sequential` to disable parallelism (useful for debugging or single-file scans).

### Cache

Pass `--no-cache` to disable the incremental scan cache. Medusa caches per-file content hashes to skip unchanged files on repeat scans.

---

## Fusion Detection

Fusion detects similar skills by computing an FxHash of:
- Skill name (lowercased)
- Content (normalized whitespace, lowercased)

Skills with identical hashes after normalization are flagged as "similar" with an edit-distance based name comparison for fuzzy name matches.

---

## Configuration

### `medusa.toml`

| Section | Key | Default | Description |
|---------|-----|---------|-------------|
| `scoring` | `complexity_weight` | 0.6 | Weight for complexity score |
| `scoring` | `value_weight` | 0.3 | Weight for value score |
| `scoring` | `keyword_weight` | 0.1 | Weight for keyword bonuses |
| `tier_thresholds` | *(optional)* | — | Override tier boundary values |

---

## Cross-Platform

- **Windows**: `medusa.exe` — native PE binary
- **macOS** (Intel + Apple Silicon): `medusa` — universal binary via `cargo build`
- **Linux** (x86_64 + ARM64): `medusa` — ELF binary

No runtime dependencies. Single-binary deployment.

---

## Build & Dependencies

### Dependencies

```
serde       — Struct serialization
serde_json  — JSON output
toml        — Config parsing
fxhash      — Fusion hash computation
walkdir     — Directory traversal
rayon       — Parallel processing
regex       — Pattern extraction
lazy_static — Regex compilation
chrono      — Timestamps for dream sessions
```

### Profile

```toml
[profile.release]
opt-level = 3
panic = "abort"
strip = true
```

**Compile time**: ~5 seconds (release, stripped)
**Binary size**: ~2MB across all platforms
