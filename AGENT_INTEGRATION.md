# Medusa Agent Integration Guide

Medusa Skill Framework outputs **pure JSON** - perfect for any agent that can run shell commands!

## Quick Integration (Any Agent)

### Hermes / OpenClaw

Add Medusa as a tool in your agent config:

```yaml
# .hermes/config.yaml (or similar)
tools:
  - name: medusa_scan
    command: "C:\\Project\\medusa-github\\target\\release\\medusa"
    args: ["scan", "${skills_path}"]
    output_format: json
    description: "Scan and audit skills with 9-tier leveling (Godlike → Poor)"
    
  - name: medusa_audit
    command: "C:\\Project\\medusa-github\\target\\release\\medusa"
    args: ["audit", "${skills_path}"]
    output_format: json
    description: "Show detailed skill audit with complexity metrics"
```

### ClaudeCode / Codex (Shell Access)

Claude/Codex can directly call Medusa via shell:

```
!cd C:\Project\medusa-github && .\target\release\medusa scan C:\Project\.opencode\skills
```

Or add as a CLI tool:
```json
{
  "name": "medusa",
  "description": "Scan SKILL.md files with audit-based ranking",
  "command": "C:\\Project\\medusa-github\\target\\release\\medusa",
  "args": ["scan", "${path}"]
}
```

### PowerShell Agent (The RIGHT Way!)

```powershell
# CORRECT way to run (NO .exe extension):
& "C:\Project\medusa-github\target\release\medusa" scan "C:\Project\.opencode\skills"

# Or with alias (Best for Agents!):
function medusa {
    & "C:\Project\medusa-github\target\release\medusa" @args
}

# Now it works like a native command:
medusa scan "C:\Project\.opencode\skills"
medusa audit "C:\Project\.opencode\skills\ai-ml"
medusa html "C:\Project\.opencode\skills" "C:\report.html"
```

### Python Agent (OpenClaw, etc.)

```python
import subprocess
import json

def scan_skills(path):
    result = subprocess.run(
        ["C:\\Project\\medusa-github\\target\\release\\medusa", "scan", path],
        capture_output=True,
        text=True
    )
    return json.loads(result.stdout)

# Usage:
data = scan_skills(r"C:\Project\.opencode\skills")
print(f"Found {data['total']} skills")
for skill in data['skills'][:3]:
    print(f"  {skill['label']} - {skill['level']} (Exp: {skill['experience']})")
```

### Node.js / JavaScript Agent

```javascript
const { exec } = require('child_process');

function scanSkills(path) {
    return new Promise((resolve, reject) => {
        exec(`"C:\\Project\\medusa-github\\target\\release\\medusa" scan "${path}"`, 
            (error, stdout, stderr) => {
                if (error) reject(error);
                else resolve(JSON.parse(stdout));
            });
    });
}

// Usage:
scanSkills("C:\\Project\\.opencode\\skills")
    .then(data => {
        console.log(`Found ${data.total} skills`);
        data.skills.slice(0, 3).forEach(s => 
            console.log(`${s.label} - ${s.level}`)
        );
    });
```

## Common Issue: "Not Recognized"

**The Problem:**
```
PS > .\target\release\medusa scan C:\path
.\target\release\medusa: The term '...' is not recognized
```

**The Fix (3 Options):**

### Option 1: Use `&` (Ampersand - Recommended)
```powershell
& "C:\Project\medusa-github\target\release\medusa" scan "C:\Project\.opencode\skills"
```

### Option 2: Use `Start-Process`
```powershell
Start-Process "C:\Project\medusa-github\target\release\medusa" -ArgumentList "scan", "C:\Project\.opencode\skills" -Wait
```

### Option 3: Create an Alias (Best for Agents!)
```powershell
# Add to your agent's startup:
function medusa {
    & "C:\Project\medusa-github\target\release\medusa" @args
}

# Now it works like a native command:
medusa scan "C:\Project\.opencode\skills"
medusa audit "C:\Project\.opencode\skills\ai-ml"
medusa --help
```

## Agent Workflow Example

```
Agent: "I need to audit all skills in the project"

↓ Agent runs:
& "C:\Project\medusa-github\target\release\medusa" audit "C:\Project\.opencode\skills"

↓ Agent parses JSON:
{
  "skills": [
    {"label": "ai-ml", "level": "Godlike", "experience": 100.0, ...},
    {"label": "agent-framework", "level": "Godlike", "experience": 98.0, ...}
  ],
  "fusion_matches": [...]
}

↓ Agent responds:
"Found 36 skills. Top tiers: 
 - ai-ml (Godlike, 100/100)
 - agent-framework (Godlike, 98/100)
..."
```

## Medusa Commands Agents Can Use

| Command | Output | Agent Use Case |
|----------|--------|---------------|
| `scan <path>` | JSON with all skills + metrics | "What skills are available?" |
| `audit <path>` | Detailed breakdown per skill | "Why is this skill Godlike?" |
| `html <path> <out>` | HTML visualization | "Generate a skill report" |
| `ab-test <path>` | Performance comparison | "Validate scan speed" |

## JSON Schema (For Agent Parsing)

```typescript
interface MedusaResult {
  skills: Array<{
    id: string;
    label: string;
    description: string;
    experience: number;  // 0-100
    level: string;       // "Godlike" | "Unique" | ... | "Poor"
    confidence: number;  // 0-1
    metrics: {
      content_length: number;
      code_blocks: number;
      step_count: number;
      tech_term_count: number;
      complexity_score: number;
      value_score: number;
    }
  }>;
  total: number;
  scan_time_ms: number;
  fusion_matches: Array<{
    skill1: string;
    skill2: string;
    similarity: number;
    match_type: string;
  }>;
  version: string;
}
```

## Quick Test (PowerShell)

```powershell
# Test if Medusa works:
& "C:\Project\medusa-github\target\release\medusa" --version

# Expected output:
# Medusa Skill Framework (MSF) v0.6.0
```

**✅ Medusa is agent-ready!** Any system that can run shell commands and parse JSON can use it. 🎮
