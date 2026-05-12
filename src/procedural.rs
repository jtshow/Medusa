use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub order: usize,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralWorkflow {
    pub name: String,
    pub category: String,
    pub steps: Vec<WorkflowStep>,
    pub source_skills: Vec<String>,
    pub usage_count: usize,
    pub created: String,
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProceduralMemory {
    pub workflows: Vec<ProceduralWorkflow>,
}

pub fn get_procedural_path(path: &Path) -> std::path::PathBuf {
    path.join(".medusa_procedural.json")
}

pub fn load_procedural(path: &Path) -> ProceduralMemory {
    let p = get_procedural_path(path);
    if p.exists() {
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        ProceduralMemory::default()
    }
}

pub fn save_procedural(path: &Path, mem: &ProceduralMemory) {
    let p = get_procedural_path(path);
    if let Ok(json) = serde_json::to_string_pretty(mem) {
        let _ = fs::write(&p, json);
    }
}

/// Check if a step sequence qualifies as a workflow (3+ consecutive steps)
fn is_workflow_candidate(content: &str) -> Vec<WorkflowStep> {
    let mut steps = Vec::new();
    let mut order = 1;
    let mut had_numbered = false;

    for line in content.lines() {
        let trimmed = line.trim();
        // Skip code blocks
        if trimmed.starts_with("```") {
            continue;
        }

        // Match numbered steps like "1. do this" or "1) do this"
        if let Some(desc) = extract_numbered_step(trimmed) {
            had_numbered = true;
            let category = categorize_step(&desc);
            steps.push(WorkflowStep { order, description: desc, category });
            order += 1;
        } else if !had_numbered {
            // Only match bullets if no numbered steps found yet
            if let Some(desc) = extract_bullet_step(trimmed) {
                let category = categorize_step(&desc);
                steps.push(WorkflowStep { order, description: desc, category });
                order += 1;
            }
        }
    }

    if steps.len() >= 3 { steps } else { Vec::new() }
}

fn extract_numbered_step(line: &str) -> Option<String> {
    let line = line.trim();
    // Match "1. text" or "1) text"
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 { return None; }
    if i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')') {
        i += 1;
        if i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        let desc = line[i..].trim().to_string();
        if !desc.is_empty() { Some(desc) } else { None }
    } else {
        None
    }
}

fn extract_bullet_step(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with("- ") || line.starts_with("* ") {
        let desc = line[2..].trim().to_string();
        if !desc.is_empty() { Some(desc) } else { None }
    } else {
        None
    }
}

fn categorize_step(text: &str) -> String {
    let t = text.to_lowercase();
    if t.contains("install") || t.contains("setup") || t.contains("configure") || t.contains("prerequisite") { "Setup".to_string()
    } else if t.contains("run") || t.contains("execute") || t.contains("start") || t.contains("deploy") { "Execution".to_string()
    } else if t.contains("test") || t.contains("verify") || t.contains("check") || t.contains("validate") || t.contains("debug") { "Verification".to_string()
    } else if t.contains("implement") || t.contains("create") || t.contains("build") || t.contains("write") || t.contains("develop") { "Implementation".to_string()
    } else if t.contains("learn") || t.contains("understand") || t.contains("review") || t.contains("study") { "Learning".to_string()
    } else if t.contains("document") || t.contains("describe") || t.contains("explain") { "Documentation".to_string()
    } else { "General".to_string() }
}

/// Extract workflows from a skill's content
pub fn extract_workflows(content: &str, skill_id: &str, path: &Path) {
    let steps = is_workflow_candidate(content);
    if steps.len() < 3 { return; }

    let mut mem = load_procedural(path);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Derive a workflow name from the skill
    let name = format!("{} Workflow", skill_id);

    // Check if already exists
    if let Some(existing) = mem.workflows.iter_mut().find(|w| w.name == name) {
        existing.usage_count += 1;
        existing.last_used = now.clone();
        if !existing.source_skills.contains(&skill_id.to_string()) {
            existing.source_skills.push(skill_id.to_string());
        }
    } else {
        mem.workflows.push(ProceduralWorkflow {
            name,
            category: steps.first().map(|s| s.category.clone()).unwrap_or_else(|| "General".to_string()),
            steps,
            source_skills: vec![skill_id.to_string()],
            usage_count: 1,
            created: now.clone(),
            last_used: now,
        });
    }

    save_procedural(path, &mem);
}

/// Extract workflows from all skills during scan
pub fn extract_workflows_from_skills(skills: &[super::Skill], contents: &HashMap<String, String>, path: &Path) {
    for skill in skills {
        if let Some(content) = contents.get(&skill.id) {
            extract_workflows(content, &skill.id, path);
        }
    }
}

pub fn get_workflows_for_skill<'a>(mem: &'a ProceduralMemory, skill_id: &str) -> Vec<&'a ProceduralWorkflow> {
    mem.workflows.iter().filter(|w| w.source_skills.contains(&skill_id.to_string())).collect()
}

pub fn print_workflow(workflow: &ProceduralWorkflow) {
    println!("  {} [{}]", workflow.name, workflow.category);
    println!("    Steps ({} total):", workflow.steps.len());
    for step in &workflow.steps {
        println!("      {}. {} ({})", step.order, step.description, step.category);
    }
    println!("    Source Skills: {}", workflow.source_skills.join(", "));
    println!("    Used {} time(s)", workflow.usage_count);
}

pub fn print_all_workflows(mem: &ProceduralMemory) {
    if mem.workflows.is_empty() {
        println!("No procedural workflows recorded.");
        println!("Run a scan to detect workflows from skill step sequences.");
        return;
    }
    println!("\n=== Procedural Memory: {} Workflows ===", mem.workflows.len());
    for w in &mem.workflows {
        println!("\n  {} [{}] — {} steps, {} source(s), used {}x",
            w.name, w.category, w.steps.len(), w.source_skills.len(), w.usage_count);
        for step in w.steps.iter().take(3) {
            println!("    {}. {} ({})", step.order, step.description, step.category);
        }
        if w.steps.len() > 3 {
            println!("    ... and {} more step(s)", w.steps.len() - 3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbered_step() {
        assert_eq!(extract_numbered_step("1. First step"), Some("First step".to_string()));
        assert_eq!(extract_numbered_step("12) Deep step"), Some("Deep step".to_string()));
        assert_eq!(extract_numbered_step("No step here"), None);
        assert_eq!(extract_numbered_step("just text"), None);
    }

    #[test]
    fn test_extract_bullet_step() {
        assert_eq!(extract_bullet_step("- Bullet item"), Some("Bullet item".to_string()));
        assert_eq!(extract_bullet_step("* Another bullet"), Some("Another bullet".to_string()));
        assert_eq!(extract_bullet_step("Not a bullet"), None);
        assert_eq!(extract_bullet_step("-"), None);
    }

    #[test]
    fn test_categorize_step() {
        assert_eq!(categorize_step("Install the package"), "Setup");
        assert_eq!(categorize_step("Run the tests"), "Execution");
        assert_eq!(categorize_step("Deploy to production"), "Execution");
        assert_eq!(categorize_step("Write the code"), "Implementation");
        assert_eq!(categorize_step("Learn about Rust"), "Learning");
        assert_eq!(categorize_step("Document the API"), "Documentation");
        assert_eq!(categorize_step("Something random"), "General");
    }

    #[test]
    fn test_is_workflow_candidate_minimum_steps() {
        let content = "1. Step one\n2. Step two\n3. Step three";
        let steps = super::is_workflow_candidate(content);
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_is_workflow_candidate_insufficient_steps() {
        let content = "1. Only one step\n2. And another";
        let steps = super::is_workflow_candidate(content);
        assert!(steps.is_empty());
    }

    #[test]
    fn test_is_workflow_candidate_skips_code_blocks() {
        let content = "\
1. First step

```python
for i in range(10):
    print(i)
```

2. Second step
3. Third step";
        let steps = super::is_workflow_candidate(content);
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_procedural_workflow_creation() {
        let wf = ProceduralWorkflow {
            name: "Test Workflow".to_string(),
            category: "Setup".to_string(),
            steps: vec![
                WorkflowStep { order: 1, description: "Install".to_string(), category: "Setup".to_string() },
                WorkflowStep { order: 2, description: "Configure".to_string(), category: "Setup".to_string() },
                WorkflowStep { order: 3, description: "Verify".to_string(), category: "Verification".to_string() },
            ],
            source_skills: vec!["test-skill".to_string()],
            usage_count: 1,
            created: "2026-01-01".to_string(),
            last_used: "2026-01-01".to_string(),
        };
        assert_eq!(wf.steps.len(), 3);
        assert_eq!(wf.category, "Setup");
    }

    #[test]
    fn test_print_workflow_does_not_panic() {
        let wf = ProceduralWorkflow {
            name: "Test Workflow".to_string(),
            category: "Setup".to_string(),
            steps: vec![
                WorkflowStep { order: 1, description: "Step 1".to_string(), category: "Setup".to_string() },
            ],
            source_skills: vec!["skill-a".to_string()],
            usage_count: 5,
            created: "2026-01-01".to_string(),
            last_used: "2026-01-05".to_string(),
        };
        print_workflow(&wf);
    }

    #[test]
    fn test_get_workflows_for_skill() {
        let mem = ProceduralMemory {
            workflows: vec![
                ProceduralWorkflow {
                    name: "WF-A".to_string(),
                    category: "Setup".to_string(),
                    steps: vec![],
                    source_skills: vec!["skill-a".to_string(), "skill-b".to_string()],
                    usage_count: 1,
                    created: "2026-01-01".to_string(),
                    last_used: "2026-01-01".to_string(),
                },
                ProceduralWorkflow {
                    name: "WF-B".to_string(),
                    category: "Execution".to_string(),
                    steps: vec![],
                    source_skills: vec!["skill-c".to_string()],
                    usage_count: 1,
                    created: "2026-01-01".to_string(),
                    last_used: "2026-01-01".to_string(),
                },
            ],
        };
        let result = get_workflows_for_skill(&mem, "skill-a");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "WF-A");

        let result = get_workflows_for_skill(&mem, "skill-c");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "WF-B");

        let result = get_workflows_for_skill(&mem, "skill-d");
        assert!(result.is_empty());
    }

    #[test]
    fn test_print_all_workflows_empty() {
        let mem = ProceduralMemory::default();
        print_all_workflows(&mem);
    }
}
