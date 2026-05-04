# Medusa Skill Framework (MSF) v0.11

![Alt text](https://i.ibb.co/twSdwzHZ/medusa.jpg)

**The world's first audit-based skill ranking system.** Medusa scans your SKILL.md files, measures actual complexity (code blocks, steps, technical terms), and automatically promotes skills through 9 tiers - just like how cooking 1 pizza vs 20+ varieties with techniques upgrades your skill level.

## What It Does.

Medusa reads your `SKILL.md` files and acts like a **technical auditor**:

1. **Measures** complexity (content length, code blocks, step-by-step instructions, technical terms)
2. **Calculates** objective experience score (60% complexity + 30% value + 10% keywords)
3. **Assigns** tier automatically (Godlike → Unique → Legendary → Mythic → Epic → Ultra Rare → Rare → Uncommon → Common → Poor)
4. **Promotes** as you improve (edit SKILL.md, next scan updates rank!)

### Pizza Example 🍕.

| Skill State | Content | Metrics | Tier |
|-------------|---------|---------|------|
| "I cook pizza" (200 chars) | 0 code, 0 steps | Complexity: 15/100 | **Poor** |
| "5 varieties" (800 chars, 5 steps) | 3 code blocks | Complexity: 35/100 | **Common** |
| "15 varieties + techniques" (3000 chars, 15 steps) | 12 tech terms | Complexity: 65/100 | **Ultra Rare** |
| "20+ varieties, ingredients, methods" (6000+ chars, 25+ terms) | 10+ code blocks | Complexity: 85+/100 | **Godlike** 🟣 |

**As you improve your skills, Medusa automatically promotes them!**

## ⚡ One-Line Install.

### Windows (Native)
```powershell
irm https://raw.githubusercontent.com/jtshow/medusa/main/install.ps1 | iex
```

### macOS / Linux
```bash
curl -sSL https://raw.githubusercontent.com/jtshow/medusa/main/install.sh | bash
```

### Build from Source (Any Platform)
```bash
# Install Rust: https://rustup.rs
git clone https://github.com/jtshow/medusa.git
cd medusa
cargo build --release
```

## Features.

| Feature | Description |
|---------|-------------|
| **9-Tier Leveling** | Godlike → Poor (based on actual complexity) |
| **Audit-Based Ranking** | Measures: length, code, steps, tech terms |
| **Automatic Promotion** | Skills rank up as YOU improve them |
| **Parallel Scanning** | Rayon-powered (46% faster, A/B tested) |
| **Fusion Detection** | Finds similar skills (name + content) |
| **HTML Visualization** | Dark-themed reports with progress bars |

## Quick Start.

### Scan Skills (JSON Output)
```bash
# Windows
medusa scan "C:\pathtoskills

# macOS/Linux
medusa scan /path/to/skills```

### Audit a Skill (See WHY it's at its tier)
```bash
medusa audit /path/to/skills/
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
medusa html /path/to/skills report.html
```
Opens beautiful dark-themed visualization with color-coded tiers!
```

## 9-Tier Leveling System.

| Tier | Range | Color |
|------|--------|--------|
| **Godlike** | 95+ | 🔴 Red-Orange-Green gradient |
| **Unique** | 90+ | 🔴 Red solid |
| **Legendary** | 85+ | 💜 Pink-Purple solid |
| **Mythic** | 80+ | 🟣 Purple solid |
| **Epic** | 75+ | 🟡 Yellow solid |
| **Ultra Rare** | 65+ | 🟢 Teal solid |
| **Rare** | 55+ | 🔵 Blue solid |
| **Uncommon** | 45+ | 🟢 Green solid |
| **Common** | 25+ | ⚪ Light Gray solid |
| **Poor** | <25 | ⚫ Dark Gray solid |

## Agent Integration ✅.

Medusa outputs **pure JSON** - perfect for any agent:

| Agent | Integration |
|-------|-------------|
| **Hermes** | Add as tool in `.hermes/config.yaml` |
| **OpenClaw** | Call via Python `subprocess` |
| **ClaudeCode/Codex** | Shell out: `medusa scan <path>` |
| **Any Agent** | Can run shell commands + parse JSON ✅ |

### Example (PowerShell Agent)
```powershell
& "C:\pathtoskillsfolder"
```

## Performance.

**A/B Test Results** (36 skills, 20 iterations):
- **Parallel** (Rayon): 190ms avg
- **Sequential**: 352ms avg
- **Speedup**: 46% faster ✅

## Cross-Platform ✅.

| Platform | Binary | Build Command |
|----------|--------|---------------|
| **Windows** | `medusa` (no .exe!) | `cargo build --release` |
| **macOS (Intel)** | `medusa` | `cargo build --release` |
| **macOS (Apple)** | `medusa` | `cargo build --release` |
| **Linux (x86_64)** | `medusa` | `cargo build --release` |

**No WSL needed!** Runs natively on all platforms.

## Update Medusa

```bash
# One-line update (pull latest + rebuild)
cd /path/to/medusa && git pull && cargo build --release
```

## Why "Audit-Based"?

Traditional systems use **static rankings** (you manually set levels).

Medusa uses **audit-based ranking**:
1. **Measures** actual skill complexity (content, code, steps, terms)
2. **Calculates** objective experience score
3. **Assigns** tier automatically
4. **Promotes** as you improve - **no manual commands needed!**

## File Structure.

```
medusa-github/
├── src/main.rs          # Core (v0.11)
├── target/release/
│   └── medusa          # Binary (NO .exe extension!)
├── Cargo.toml          # 7 minimal deps
├── README.md           # This file
├── LICENSE             # GNU GPLv3
├── build.sh            # Linux/macOS build
├── build.bat           # Windows build
├── install.sh          # One-line installer
├── install.ps1         # PowerShell installer
└── AGENT_INTEGRATION.md  # Agent hook guide
```


---

**Built with Rust 🦀 + Rayon ⚡ + Regex 🔍**  
**Cross-platform**: Windows 🪟 | macOS 🍎 | Linux 🐧**
