# Medusa Skill Framework (MSF) v0.6.0

Ultra-fast skill scanner with **audit-based ranking**, automatic promotion, and 9-tier leveling system.

## ⚡ One-Line Install

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/your-repo/medusa/main/install.ps1 | iex
```

### Windows (Command Prompt)
```batch
curl -SL https://raw.githubusercontent.com/your-repo/medusa/main/install.bat -o install.bat && install.bat
```

### macOS / Linux
```bash
curl -sSL https://raw.githubusercontent.com/your-repo/medusa/main/install.sh | bash
```

### Build from Source (Any Platform)

**Step 1: Install Rust**
- Windows: `irm https://win.rustup.rs/x86_64 | iex` (or download from https://rustup.rs)
- macOS/Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

**Step 2: Clone & Build**
```bash
# All platforms (Windows CMD/PowerShell, macOS, Linux):
git clone https://github.com/your-repo/medusa.git
cd medusa
cargo build --release
```

**Binary Location:**
| Platform | Binary Path |
|----------|--------------|
| **Windows** | `target\release\medusa.exe` |
| **macOS/Linux** | `target/release/medusa` |

**Step 3: Run**
```bash
# Windows (CMD)
.\target\release\medusa.exe --help

# Windows (PowerShell)
.\target\release\medusa.exe --help

# macOS/Linux
./target/release/medusa --help
```

| Feature | Description | Performance |
|---------|-------------|-------------|
| **Audit-Based Ranking** | Measures complexity, value, technical depth | 9-tier system |
| **Automatic Promotion** | Skills rank up as they improve | No manual needed |
| **Parallel Scanning** | Rayon-powered concurrent processing | 46% faster (A/B tested) |
| **Fusion Detection** | Finds similar skills (name + content) | FxHash-powered |
| **HTML Visualization** | Dark-themed reports with progress bars | Interactive |
| **A/B Test Framework** | Validate performance claims scientifically | Statistical rigor |

## Usage

### Scan Skills (JSON Output)

```bash
# Windows (CMD)
.\target\release\medusa.exe scan C:\path\to\skills

# Windows (PowerShell)
.\target\release\medusa.exe scan C:\path\to\skills

# macOS/Linux
./target/release/medusa scan /path/to/skills
```

### Audit a Skill (See WHY it's at its tier)

```bash
# Windows
.\target\release\medusa.exe audit C:\path\to\skills

# macOS/Linux
./target/release/medusa audit /path/to/skills
```

**Example Output:**
```
=== Medusa Skill Audit Report ===

Skill: ai-ml (ai-ml), level: Godlike
  Experience: 100.0/100
  Confidence: 75%
  Metrics:
    - Content Length: 5966 chars
    - Code Blocks: 15
    - Step Instructions: 0
    - Technical Terms: 26
    - Complexity Score: 80.0/100
    - Value Score: 90.0/100
```

### Generate HTML Report

```bash
# Windows (CMD)
.\target\release\medusa.exe html C:\path\to\skills C:\path\to\report.html

# Windows (PowerShell)
.\target\release\medusa.exe html C:\path\to\skills C:\path\to\report.html

# macOS/Linux
./target/release/medusa html /path/to/skills /path/to/report.html
```

Opens a beautiful dark-themed visualization with:
- Skill bars showing experience levels
- Color-coded tiers (Godlike = Purple gradient, Unique = Orange gradient, etc.)
- Detailed metrics for each skill
- Fusion detection (similar skills)

### Run A/B Test (Validate Performance)

```bash
# Windows (CMD)
.\target\release\medusa.exe ab-test C:\path\to\skills --iterations 20

# Windows (PowerShell)
.\target\release\medusa.exe ab-test C:\path\to\skills --iterations 20

# macOS/Linux
./target/release/medusa ab-test /path/to/skills --iterations 20
```

**Example Output:**
```
Running A/B Test: Parallel vs Sequential Scan
Path: /path/to/skills
Iterations: 20

Hypothesis: Parallel scanning is faster than sequential
Primary metric: scan_time_ms

Iteration 1: Parallel=178ms, Sequential=362ms
Iteration 2: Parallel=187ms, Sequential=325ms
...
Iteration 20: Parallel=204ms, Sequential=391ms

=== A/B Test Results ===
Parallel avg: 190.00ms
Sequential avg: 352.00ms
✅ Parallel is 46.0% faster
```

## 9-Tier Leveling System

| Tier | Range | Color | Background |
|------|--------|--------|-------------|
| **Godlike** | 95+ | 🟣 Purple | Gradient |
| **Unique** | 90+ | 🟠 Orange | Gradient |
| **Legendary** | 85+ | 🟠 Orange-Red | Gradient |
| **Mythic** | 80+ | 🟣 Pink | Gradient |
| **Epic** | 75+ | 🟣 Purple | Solid |
| **Ultra Rare** | 65+ | 🩷 Pink | Solid |
| **Rare** | 55+ | 🟠 Orange | Solid |
| **Uncommon** | 45+ | 🔵 Blue | Solid |
| **Common** | 25+ | 🟢 Green | Solid |
| **Poor** | <25 | ⚪ Gray | Solid |

## Commands

```bash
medusa --help
```

Output:
```
Medusa Skill Framework (MSF) v0.6.0 - Audit-Based Ranking
Usage: medusa <command> [options]

Commands:
  scan <path>              Scan skills with audit-based ranking
    --sequential           Use sequential scanning (no Rayon)

  html <path> <output>   Generate HTML visualization
    --sequential           Use sequential scanning

  ab-test <path>          Run A/B test (parallel vs sequential)
    --iterations N         Number of test iterations (default: 10)

  audit <path>            Show detailed skill audit report

Options:
  --help, -h              Show this help message
  --version, -v           Show version
```

## How It Works

```
SKILL.md files
    ↓
[WalkDir] Scan filesystem (max depth 4)
    ↓
[Rayon] Parallel processing (46% faster, optional)
    ↓
[Regex] Extract YAML frontmatter
    ↓
[Audit] Measure complexity (length, code, steps, terms)
    ↓
[Score] Calculate experience (60% complexity + 30% value)
    ↓
[Rank] Assign tier (Godlike → Poor)
    ↓
[Fusion] Detect similar skills (FxHash)
    ↓
[Output] JSON / HTML visualization
```

## Cross-Platform Support ✅

| Platform | Binary | Build Command |
|----------|--------|---------------|
| **Windows** | `medusa.exe` | `cargo build --release` |
| **macOS (Intel)** | `medusa` | `cargo build --release` |
| **macOS (Apple Silicon)** | `medusa` | `cargo build --release` |
| **Linux (x86_64)** | `medusa` | `cargo build --release` |
| **Linux (ARM64)** | `medusa` | `cargo build --release` |

**No WSL required!** Runs natively on all platforms.

```
medusa/
├── src/
│   └── main.rs          # Main source (500+ lines)
├── target/
│   └── release/
│       └── medusa       # Compiled binary
├── Cargo.toml          # Dependencies (minimal: 7 deps)
├── README.md           # This file
└── .medusa_state.json  # Promotion state (auto-created)
```

## Dependencies (Minimal)

```
serde = "1.0"        # Struct serialization
serde_json = "1.0"    # JSON output
walkdir = "2.5"       # Directory traversal
rayon = "1.10"        # Parallel processing
regex = "1.10"        # Pattern extraction
lazy_static = "1.4"   # Regex compilation
```

## Examples

### Example 1: Scan Your Skills

```bash
# Windows
.\target\release\medusa.exe scan C:\Project\.opencode\skills

# macOS/Linux
./target/release/medusa scan ~/.hermes/skills
```

### Example 2: Audit a Specific Skill

```bash
# Windows
.\target\release\medusa.exe audit C:\Project\.opencode\skills\ai-ml

# macOS/Linux
./target/release/medusa audit ~/.hermes/skills/ai-ml
```

Shows detailed breakdown:
- Why it's ranked "Godlike"
- Content length, code blocks, step count
- Technical term density
- Complexity and value scores

### Example 3: Generate Beautiful Report

```bash
# Windows
.\target\release\medusa.exe html C:\Project\.opencode\skills C:\Project\medusa\report.html

# macOS/Linux
./target/release/medusa html ~/.hermes/skills ~/report.html
```

### Example 2: Audit a Specific Skill
```bash
# Shows WHY it's at its tier
medusa audit /path/to/skills/ai-ml
```

**Example Output:**
```
=== Medusa Skill Audit Report ===

Skill: ai-ml (ai-ml), level: Godlike
  Experience: 100.0/100
  Confidence: 75%
  Metrics:
    - Content Length: 5966 chars
    - Code Blocks: 15
    - Step Instructions: 0
    - Technical Terms: 26
    - Complexity Score: 80.0/100
    - Value Score: 90.0/100
```

### Example 3: Generate Beautiful Report
```bash
# Windows
.\target\release\medusa.exe html C:\path\to\skills report.html

# Mac/Linux
./target/release/medusa html /path/to/skills report.html
```

Opens in browser with:
- Dark theme (hacker style)
- Color-coded skill cards
- Experience progress bars
- Fusion detection section

## Performance

**A/B Test Results** (36 skills, 20 iterations):
- **Parallel** (Rayon): 190ms average
- **Sequential**: 352ms average
- **Speedup**: 46% faster

**Scalability**:
- 36 skills scanned in ~150ms
- Linear scaling with Rayon parallelization
- Memory-efficient (no unnecessary copies)

## Quick Test (Verify Installation)

After installation, verify it works:

```bash
# Windows (CMD)
.\target\release\medusa.exe --version

# Windows (PowerShell)
.\target\release\medusa.exe --version

# macOS/Linux
./target/release/medusa --version
```

**Expected output:**
```
Medusa Skill Framework (MSF) v0.6.0
```

### Test Scan
```bash
# Windows: medusa scan C:\Project\.opencode\skills

# macOS/Linux:
./target/release/medusa scan ~/.hermes/skills
```

Should output JSON with skills audit.

Traditional skill systems use **static rankings** (you manually set the level).

Medusa uses **audit-based ranking**:
1. **Measures** actual skill complexity (content, code, steps, terms)
2. **Calculates** objective experience score (60% complexity + 30% value)
3. **Assigns** tier automatically (Godlike → Poor)
4. **Promotes** as you improve (just edit SKILL.md, next scan updates!)

**No manual promotion commands needed!**


## Version History

- **v0.6.0** (Current): 9-tier leveling system (Godlike → Poor)
- **v0.5.0**: Rank promotion system
- **v0.4.0**: CLI improvements, A/B test framework
- **v0.3.0**: Fusion detection, HTML visualization
- **v0.2.0**: Parallel scanning with Rayon
- **v0.1.0**: Initial release

---

**Built with Rust 🦀 + Rayon ⚡ + Regex 🔍**
