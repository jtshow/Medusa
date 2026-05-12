use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SubAudit {
    pub agent_name: String,
    pub score: f64,
    pub max_score: f64,
    pub findings: Vec<String>,
    #[allow(dead_code)]
    pub details: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct OrchestratedAudit {
    pub skill_id: String,
    pub overall_score: f64,
    pub sub_audits: Vec<SubAudit>,
    #[allow(dead_code)]
    pub synthesized_summary: Vec<String>,
}

pub trait AuditAgent {
    fn name(&self) -> &str;
    fn assess(&self, metrics: &super::SkillMetrics, context: &super::SkillContext, content: &str) -> SubAudit;
}

pub struct DocQualityAgent;
pub struct CodeQualityAgent;
pub struct DependencyAgent;
pub struct LearningValueAgent;

impl AuditAgent for DocQualityAgent {
    fn name(&self) -> &str { "Documentation Quality" }
    fn assess(&self, metrics: &super::SkillMetrics, _context: &super::SkillContext, content: &str) -> SubAudit {
        let mut score = 0.0;
        let mut findings = Vec::new();
        let mut details = HashMap::new();

        // Content length score (max 25)
        let length_score = (metrics.content_length as f64 / 200.0).min(25.0);
        score += length_score;
        details.insert("content_length".to_string(), length_score);
        if metrics.content_length < 1000 {
            findings.push("Content is short (<1000 chars). Expand to improve depth.".to_string());
        } else if metrics.content_length >= 5000 {
            findings.push("Content is comprehensive (>5000 chars).".to_string());
        }

        // Step instructions score (max 25)
        let step_score = (metrics.step_count as f64 * 2.5).min(25.0);
        score += step_score;
        details.insert("step_instructions".to_string(), step_score);
        if metrics.step_count < 3 {
            findings.push("Few or no step instructions. Add structured guidance.".to_string());
        } else if metrics.step_count >= 10 {
            findings.push("Excellent step-by-step coverage.".to_string());
        }

        // Heading structure score (max 25)
        let heading_count = content.lines().filter(|l| l.trim().starts_with("##") || l.trim().starts_with("###")).count();
        let heading_score = (heading_count as f64 * 3.0).min(25.0);
        score += heading_score;
        details.insert("heading_structure".to_string(), heading_score);
        if heading_count < 3 {
            findings.push("Limited section structure. Use headings (## / ###) to organize.".to_string());
        } else if heading_count >= 8 {
            findings.push("Well-structured with clear sections.".to_string());
        }

        // Readability / formatting (max 25)
        let total_lines = content.lines().count().max(1) as f64;
        let avg_line_len = metrics.content_length as f64 / total_lines;
        let readability_score = if avg_line_len > 30.0 && avg_line_len < 120.0 { 20.0 } else { 10.0 };
        score += readability_score;
        details.insert("readability".to_string(), readability_score);
        if avg_line_len < 30.0 {
            findings.push("Lines are very short — may indicate sparse content.".to_string());
        } else if avg_line_len > 200.0 {
            findings.push("Very long lines — consider breaking into readable paragraphs.".to_string());
        }

        SubAudit { agent_name: self.name().to_string(), score, max_score: 100.0, findings, details }
    }
}

impl AuditAgent for CodeQualityAgent {
    fn name(&self) -> &str { "Code Quality" }
    fn assess(&self, metrics: &super::SkillMetrics, context: &super::SkillContext, content: &str) -> SubAudit {
        let mut score = 0.0;
        let mut findings = Vec::new();
        let mut details = HashMap::new();

        // Code blocks quantity (max 35)
        let code_score = (metrics.code_blocks as f64 * 3.5).min(35.0);
        score += code_score;
        details.insert("code_blocks".to_string(), code_score);
        if metrics.code_blocks == 0 {
            findings.push("No code examples. Add practical code blocks for +5 complexity each.".to_string());
        } else if metrics.code_blocks >= 10 {
            findings.push(format!("Strong code coverage ({} blocks).", metrics.code_blocks));
        }

        // Language diversity in code fences (max 25)
        let mut languages: Vec<String> = Vec::new();
        let mut in_block = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") && !in_block {
                in_block = true;
                let lang = trimmed.trim_start_matches("```").trim().to_string();
                if !lang.is_empty() && !languages.contains(&lang) {
                    languages.push(lang);
                }
            } else if trimmed.starts_with("```") && in_block {
                in_block = false;
            }
        }
        let lang_score = (languages.len() as f64 * 6.0).min(25.0);
        score += lang_score;
        details.insert("language_diversity".to_string(), lang_score);
        if languages.len() >= 3 {
            findings.push(format!("Multi-language examples across {} languages.", languages.len()));
        } else if metrics.code_blocks > 0 && languages.len() <= 1 {
            findings.push("Consider examples in multiple languages.".to_string());
        }

        // Code-to-text ratio (max 20)
        let code_line_count = count_code_lines(content);
        let total_lines = content.lines().count() as f64;
        let code_ratio = if total_lines > 0.0 { code_line_count as f64 / total_lines } else { 0.0 };
        let ratio_score = if code_ratio > 0.1 && code_ratio < 0.6 { 20.0 } else if code_ratio > 0.0 { 10.0 } else { 0.0 };
        score += ratio_score;
        details.insert("code_ratio".to_string(), ratio_score);
        if code_ratio > 0.6 {
            findings.push("Very high code-to-text ratio. Add more explanatory text.".to_string());
        } else if code_ratio < 0.05 && metrics.code_blocks > 0 {
            findings.push("Low code-to-text ratio for the number of blocks.".to_string());
        }

        // Gap impact (max 20)
        let gap_penalty = context.gaps.iter().filter(|g| g.to_lowercase().contains("code")).count() as f64 * 5.0;
        let gap_score = (20.0 - gap_penalty).max(0.0);
        score += gap_score;
        details.insert("gap_penalty".to_string(), gap_penalty);
        if gap_penalty > 0.0 {
            findings.push("Code-related gaps detected in skill context.".to_string());
        }

        SubAudit { agent_name: self.name().to_string(), score, max_score: 100.0, findings, details }
    }
}

impl AuditAgent for DependencyAgent {
    fn name(&self) -> &str { "Dependency Health" }
    fn assess(&self, metrics: &super::SkillMetrics, context: &super::SkillContext, _content: &str) -> SubAudit {
        let mut score = 0.0;
        let mut findings = Vec::new();
        let mut details = HashMap::new();

        // Dependency count & health (max 35)
        let dep_count = context.dependencies.len();
        let dep_score = (dep_count as f64 * 7.0).min(35.0);
        score += dep_score;
        details.insert("dependency_count".to_string(), dep_score);
        if dep_count == 0 {
            findings.push("No dependencies mapped. Skills may be isolated.".to_string());
        } else if dep_count >= 3 {
            findings.push(format!("Good dependency connectivity ({} deps).", dep_count));
        }

        // Fusion risk (max 25)
        let fusion_count = context.fusion_opportunities.len();
        let fusion_penalty = (fusion_count as f64 * 5.0).min(25.0);
        let fusion_score = 25.0 - fusion_penalty;
        score += fusion_score;
        details.insert("fusion_risk".to_string(), fusion_penalty);
        if fusion_count > 2 {
            findings.push(format!("High fusion risk with {} similar skills. Consider consolidation.", fusion_count));
        } else if fusion_count > 0 {
            findings.push(format!("{} fusion opportunities detected.", fusion_count));
        } else {
            findings.push("No similar skills detected — good differentiation.".to_string());
        }

        // Tech term diversity (max 25)
        let term_score = (metrics.tech_term_count as f64 * 1.5).min(25.0);
        score += term_score;
        details.insert("tech_term_diversity".to_string(), term_score);
        if metrics.tech_term_count < 5 {
            findings.push("Low technical vocabulary. Add domain-specific terms.".to_string());
        } else if metrics.tech_term_count >= 15 {
            findings.push("Rich technical vocabulary across the skill.".to_string());
        }

        // Content-dependency alignment (max 15)
        let alignment_score = if dep_count > 0 && metrics.tech_term_count > 5 { 15.0 } else if dep_count > 0 { 8.0 } else { 3.0 };
        score += alignment_score;
        details.insert("alignment".to_string(), alignment_score);

        SubAudit { agent_name: self.name().to_string(), score, max_score: 100.0, findings, details }
    }
}

impl AuditAgent for LearningValueAgent {
    fn name(&self) -> &str { "Learning Value" }
    fn assess(&self, metrics: &super::SkillMetrics, context: &super::SkillContext, _content: &str) -> SubAudit {
        let mut score = 0.0;
        let mut findings = Vec::new();
        let mut details = HashMap::new();

        // Gap severity (max 30)
        let gap_count = context.gaps.len();
        let gap_penalty = (gap_count as f64 * 10.0).min(30.0);
        let gap_score = 30.0 - gap_penalty;
        score += gap_score;
        details.insert("gap_penalty".to_string(), gap_penalty);
        if gap_count >= 3 {
            findings.push(format!("{} gaps identified — prioritize addressing them.", gap_count));
        } else if gap_count == 0 {
            findings.push("No critical gaps — skill is well-rounded.".to_string());
        }

        // Complexity value (max 30)
        let comp_score = (metrics.complexity_score * 0.3).min(30.0);
        score += comp_score;
        details.insert("complexity_value".to_string(), comp_score);
        if metrics.complexity_score >= 80.0 {
            findings.push("High complexity score indicates strong depth.".to_string());
        } else if metrics.complexity_score < 40.0 {
            findings.push("Low complexity — consider expanding coverage.".to_string());
        }

        // Improvement history (max 20)
        let history_count = context.improvement_history.len();
        let history_score = (history_count as f64 * 5.0).min(20.0);
        score += history_score;
        details.insert("improvement_history".to_string(), history_score);
        if history_count > 0 {
            findings.push(format!("{} improvement records — skill is actively maintained.", history_count));
        }

        // Experience confidence (max 20)
        let conf_score = (metrics.value_score * 0.2).min(20.0);
        score += conf_score;
        details.insert("confidence_value".to_string(), conf_score);
        if metrics.value_score >= 80.0 {
            findings.push("High value score — skill covers practical needs well.".to_string());
        }

        SubAudit { agent_name: self.name().to_string(), score, max_score: 100.0, findings, details }
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct UpgradeSuggestion {
    pub skill_id: String,
    pub suggestion: String,
    pub impact: f64,
    pub confidence: f64,
    pub category: String,
    pub priority: f64,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct BackgroundSkillUpgradeAgent {
    pub upgrade_suggestions: Vec<UpgradeSuggestion>,
    last_run: Option<u64>,
    pub skills_upgraded: usize,
}

#[allow(dead_code)]
impl BackgroundSkillUpgradeAgent {
    pub fn new() -> Self {
        BackgroundSkillUpgradeAgent {
            upgrade_suggestions: Vec::new(),
            last_run: None,
            skills_upgraded: 0,
        }
    }

    /// Analyze a skill and generate upgrade suggestions.
    pub fn analyze_for_upgrades(&mut self, skill: &super::Skill, content: &str) -> Vec<UpgradeSuggestion> {
        let mut suggestions = Vec::new();

        // Check for missing code examples
        if skill.metrics.code_blocks < 5 {
            suggestions.push(UpgradeSuggestion {
                skill_id: skill.id.clone(),
                suggestion: format!("Add {} more code examples (currently {}). Each adds +5 complexity.",
                    5 - skill.metrics.code_blocks, skill.metrics.code_blocks),
                impact: (5 - skill.metrics.code_blocks) as f64 * 5.0 * 0.6, // 60% weight
                confidence: if skill.metrics.code_blocks >= 3 { 0.7 } else { 0.9 },
                category: "code_examples".to_string(),
                priority: if skill.metrics.code_blocks == 0 { 1.0 } else { 0.7 },
            });
        }

        // Check for missing step instructions
        if skill.metrics.step_count < 10 {
            suggestions.push(UpgradeSuggestion {
                skill_id: skill.id.clone(),
                suggestion: format!("Add {} more step instructions (currently {}). Target 10+ for full points.",
                    10 - skill.metrics.step_count, skill.metrics.step_count),
                impact: (10 - skill.metrics.step_count) as f64 * 2.0 * 0.6,
                confidence: 0.8,
                category: "step_instructions".to_string(),
                priority: if skill.metrics.step_count < 3 { 0.9 } else { 0.6 },
            });
        }

        // Check for low content length
        if skill.metrics.content_length < 3000 {
            let deficit = 3000 - skill.metrics.content_length;
            suggestions.push(UpgradeSuggestion {
                skill_id: skill.id.clone(),
                suggestion: format!("Expand content by {} chars (currently {}). Every 100 chars = +1 complexity.",
                    deficit, skill.metrics.content_length),
                impact: (deficit as f64 / 100.0).min(30.0) * 0.6,
                confidence: 0.85,
                category: "content_length".to_string(),
                priority: if skill.metrics.content_length < 1000 { 1.0 } else { 0.6 },
            });
        }

        // Check for missing technical terms
        if skill.metrics.tech_term_count < 12 {
            let deficit = 12 - skill.metrics.tech_term_count;
            suggestions.push(UpgradeSuggestion {
                skill_id: skill.id.clone(),
                suggestion: format!("Add {} more technical terms (currently {}). Target 12+ for full points.",
                    deficit, skill.metrics.tech_term_count),
                impact: (deficit as f64 * 2.5).min(25.0) * 0.6,
                confidence: 0.75,
                category: "tech_terms".to_string(),
                priority: 0.7,
            });
        }

        // Check for missing description keywords
        let desc_lower = skill.description.to_lowercase();
        let high_value_keywords = ["advanced", "expert", "senior", "security", "rust", "kubernetes", "machine learning", "ai"];
        let mut missing_keywords = Vec::new();
        for kw in &high_value_keywords {
            if !desc_lower.contains(kw) {
                missing_keywords.push(*kw);
            }
        }
        if !missing_keywords.is_empty() && missing_keywords.len() <= 3 {
            suggestions.push(UpgradeSuggestion {
                skill_id: skill.id.clone(),
                suggestion: format!("Consider adding keywords to description: {}. These boost experience scoring.",
                    missing_keywords.join(", ")),
                impact: 3.0,
                confidence: 0.6,
                category: "keywords".to_string(),
                priority: 0.4,
            });
        }

        // Analyze code content for potential additions
        if !content.is_empty() {
            let content_lower = content.to_lowercase();

            // Detect if skill could benefit from additional languages
            let existing_langs = ["python", "rust", "javascript", "typescript", "java", "go", "c++"]
                .iter()
                .filter(|lang| content_lower.contains(*lang))
                .count();

            if existing_langs < 2 && skill.metrics.code_blocks > 0 {
                suggestions.push(UpgradeSuggestion {
                    skill_id: skill.id.clone(),
                    suggestion: "Add code examples in additional programming languages to improve language diversity score.".to_string(),
                    impact: 5.0,
                    confidence: 0.7,
                    category: "language_diversity".to_string(),
                    priority: 0.5,
                });
            }
        }

        // Analyze gaps from existing context
        for gap in &skill.context.gaps {
            let gap_lower = gap.to_lowercase();
            if gap_lower.contains("step") && skill.metrics.step_count < 10 {
                suggestions.push(UpgradeSuggestion {
                    skill_id: skill.id.clone(),
                    suggestion: format!("Address identified gap: '{}'. Add structured step-by-step guidance.", gap),
                    impact: 8.0,
                    confidence: 0.9,
                    category: "gap_resolution".to_string(),
                    priority: 0.8,
                });
            }
            if gap_lower.contains("code") && skill.metrics.code_blocks < 5 {
                suggestions.push(UpgradeSuggestion {
                    skill_id: skill.id.clone(),
                    suggestion: format!("Address identified gap: '{}'. Add more practical code examples.", gap),
                    impact: 10.0,
                    confidence: 0.85,
                    category: "gap_resolution".to_string(),
                    priority: 0.85,
                });
            }
        }

        // De-duplicate suggestions (same skill_id + category)
        let mut seen: HashMap<(String, String), usize> = HashMap::new();
        suggestions.retain(|s| {
            let key = (s.skill_id.clone(), s.category.clone());
            if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(key) {
                e.insert(0);
                true
            } else {
                false
            }
        });

        // Sort by priority (highest first)
        suggestions.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        // Store for later retrieval
        self.upgrade_suggestions.extend(suggestions.clone());
        self.last_run = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

        suggestions
    }

    /// Generate a comprehensive upgrade plan for a skill.
    pub fn generate_upgrade_plan(&self, skill_id: &str) -> Vec<&UpgradeSuggestion> {
        self.upgrade_suggestions
            .iter()
            .filter(|s| s.skill_id == skill_id)
            .collect()
    }

    /// Apply an upgrade suggestion by recording it as an improvement record.
    pub fn apply_suggestion(&mut self, _skill: &super::Skill, suggestion: &UpgradeSuggestion) -> super::ImprovementRecord {
        self.skills_upgraded += 1;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        super::ImprovementRecord {
            date: now,
            action: format!("Applied upgrade: {}", suggestion.suggestion),
            impact: format!("Expected +{:.1} experience points", suggestion.impact),
            evidence: format!("Category: {}, Confidence: {:.0}%", suggestion.category, suggestion.confidence * 100.0),
        }
    }

    /// Get summary statistics for upgrade suggestions.
    pub fn get_summary(&self) -> (usize, usize) {
        let unique_skills: std::collections::HashSet<&String> = self.upgrade_suggestions.iter().map(|s| &s.skill_id).collect();
        (unique_skills.len(), self.upgrade_suggestions.len())
    }
}

impl Default for BackgroundSkillUpgradeAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn count_code_lines(content: &str) -> usize {
    let mut count = 0;
    let mut in_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if in_block {
            count += 1;
        }
    }
    count
}

pub fn run_orchestrated_audit(skill: &super::Skill, content: &str) -> OrchestratedAudit {
    let agents: Vec<Box<dyn AuditAgent>> = vec![
        Box::new(DocQualityAgent),
        Box::new(CodeQualityAgent),
        Box::new(DependencyAgent),
        Box::new(LearningValueAgent),
    ];

    let mut sub_audits = Vec::new();
    for agent in agents {
        sub_audits.push(agent.assess(&skill.metrics, &skill.context, content));
    }

    // Synthesize overall score (weighted average)
    let weights: Vec<f64> = vec![0.25, 0.30, 0.20, 0.25];
    let overall: f64 = sub_audits.iter().zip(&weights)
        .map(|(sa, w)| (sa.score / sa.max_score) * w)
        .sum::<f64>() * 100.0;

    let mut summary = Vec::new();
    for sa in &sub_audits {
        let status = if sa.score >= 70.0 { "✅" } else if sa.score >= 40.0 { "⚠️" } else { "❌" };
        summary.push(format!("{} {}: {:.0}/100", status, sa.agent_name, sa.score));
    }
    let overall_status = if overall >= 70.0 { "✅ Good" } else if overall >= 40.0 { "⚠️ Needs Work" } else { "❌ Poor" };
    summary.push(format!("Overall: {} ({:.0}/100)", overall_status, overall));

    OrchestratedAudit {
        skill_id: skill.id.clone(),
        overall_score: overall,
        sub_audits,
        synthesized_summary: summary,
    }
}

pub fn run_orchestrated_audit_all(skills: &[super::Skill], contents: &HashMap<String, String>) -> Vec<OrchestratedAudit> {
    skills.iter().map(|s| {
        let content = contents.get(&s.id).map(|c| c.as_str()).unwrap_or("");
        run_orchestrated_audit(s, content)
    }).collect()
}

pub fn print_orchestrated_audit(audit: &OrchestratedAudit) {
    println!("\n=== Orchestrated Audit: {} ===", audit.skill_id);
    println!("  Overall Score: {:.1}/100", audit.overall_score);

    let overall_status = if audit.overall_score >= 70.0 { "Good" } else if audit.overall_score >= 40.0 { "Needs Work" } else { "Poor" };
    println!("  Status: {}", overall_status);

    for sa in &audit.sub_audits {
        let bar = make_bar(sa.score, 100.0, 20);
        let status = if sa.score >= 70.0 { "✓" } else if sa.score >= 40.0 { "~" } else { "✗" };
        println!("\n  [{}] {}: {:.0}/100", status, sa.agent_name, sa.score);
        println!("    {}", bar);
        for finding in &sa.findings {
            println!("    - {}", finding);
        }
    }
    println!();
}

fn make_bar(value: f64, max: f64, width: usize) -> String {
    let filled = ((value / max) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("[{}>{}]", "=".repeat(filled), " ".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SkillMetrics;

    /// Helper to create a SkillMetrics for testing.
    fn test_metrics(content_length: usize, code_blocks: usize, step_count: usize, tech_term_count: usize) -> SkillMetrics {
        SkillMetrics {
            content_length,
            code_blocks,
            step_count,
            tech_term_count,
            complexity_score: 50.0,
            value_score: 50.0,
        }
    }

    /// Helper to create an empty SkillContext.
    fn empty_context() -> crate::SkillContext {
        crate::SkillContext {
            dependencies: Vec::new(),
            fusion_opportunities: Vec::new(),
            improvement_history: Vec::new(),
            gaps: Vec::new(),
        }
    }

    // ─── DocQualityAgent ──────────────────────────────────────────────────────

    #[test]
    fn test_doc_quality_agent_comprehensive() {
        let agent = DocQualityAgent;
        let content = "\
## Introduction

This is comprehensive documentation.

### Setup

1. Install the package
2. Configure settings
3. Run tests

### Usage

```python
import mylib
mylib.run()
```

### Advanced Topics

- Performance tuning
- Security considerations
- Deployment strategies";

        let audit = agent.assess(&test_metrics(5000, 2, 6, 10), &empty_context(), content);
        assert!(audit.score > 50.0, "Comprehensive doc should score well, got {}", audit.score);
        assert!(audit.score <= 100.0);
        assert!(audit.findings.iter().any(|f| f.contains("comprehensive")));
    }

    #[test]
    fn test_doc_quality_agent_minimal() {
        let agent = DocQualityAgent;
        let content = "Hello world.";

        let audit = agent.assess(&test_metrics(12, 0, 0, 0), &empty_context(), content);
        assert!(audit.score < 50.0, "Minimal doc should score low, got {}", audit.score);
        assert!(audit.findings.iter().any(|f| f.contains("short")));
        assert!(audit.findings.iter().any(|f| f.contains("sparse")));
    }

    #[test]
    fn test_doc_quality_agent_score_range() {
        let agent = DocQualityAgent;
        let content = "## Title\n\nSome content.";
        let audit = agent.assess(&test_metrics(100, 0, 0, 0), &empty_context(), content);
        assert!(audit.score >= 0.0 && audit.score <= 100.0, "Score out of range: {}", audit.score);
    }

    #[test]
    fn test_doc_quality_agent_max_content_length() {
        let agent = DocQualityAgent;
        let long_content = "x".repeat(6000);
        let audit = agent.assess(&test_metrics(6000, 0, 0, 0), &empty_context(), &long_content);
        // Length contribution should be capped at 25
        assert!(audit.score <= 100.0);
    }

    // ─── CodeQualityAgent ─────────────────────────────────────────────────────

    #[test]
    fn test_code_quality_agent_with_code_blocks() {
        let agent = CodeQualityAgent;
        let content = "\
```python
def hello():
    print('hello')
```

Some text.

```rust
fn main() {}
```

More text.

```javascript
console.log('hi');
```";

        let audit = agent.assess(&test_metrics(500, 3, 5, 8), &empty_context(), content);
        assert!(audit.score > 30.0, "Should reward multiple code blocks, got {}", audit.score);
        assert!(audit.findings.iter().any(|f| f.contains("Multi-language")));
    }

    #[test]
    fn test_code_quality_agent_no_code() {
        let agent = CodeQualityAgent;
        let content = "Just text, no code at all.";

        let audit = agent.assess(&test_metrics(200, 0, 0, 0), &empty_context(), content);
        assert!(audit.findings.iter().any(|f| f.contains("No code examples")));
    }

    #[test]
    fn test_code_quality_agent_single_language() {
        let agent = CodeQualityAgent;
        let content = "\
```python
x = 1
```

```python
y = 2
```

```python
z = 3
```";
        let audit = agent.assess(&test_metrics(300, 3, 0, 2), &empty_context(), content);
        assert!(audit.findings.iter().any(|f| f.contains("multiple languages")));
    }

    #[test]
    fn test_code_quality_agent_gap_penalty() {
        let agent = CodeQualityAgent;
        let mut context = empty_context();
        context.gaps.push("Add more code examples".to_string());
        context.gaps.push("Add technical terms".to_string());

        let audit = agent.assess(&test_metrics(100, 1, 0, 0), &context, "no code here");
        // Gaps with "code" in them should reduce the gap_score
        assert!(audit.score < 100.0);
    }

    // ─── DependencyAgent ──────────────────────────────────────────────────────

    #[test]
    fn test_dependency_agent_with_deps() {
        let agent = DependencyAgent;
        let mut context = empty_context();
        context.dependencies.push(crate::SkillDep {
            name: "rust".to_string(),
            relationship: "uses".to_string(),
            context: "core language".to_string(),
        });
        context.dependencies.push(crate::SkillDep {
            name: "docker".to_string(),
            relationship: "deploys".to_string(),
            context: "containerization".to_string(),
        });
        context.dependencies.push(crate::SkillDep {
            name: "kubernetes".to_string(),
            relationship: "orchestrates".to_string(),
            context: "container management".to_string(),
        });

        let audit = agent.assess(&test_metrics(1000, 5, 10, 15), &context, "content");
        assert!(audit.score > 40.0, "Should score well with dependencies, got {}", audit.score);
        assert!(audit.findings.iter().any(|f| f.contains("Good dependency connectivity")));
    }

    #[test]
    fn test_dependency_agent_no_deps() {
        let agent = DependencyAgent;
        let audit = agent.assess(&test_metrics(1000, 5, 10, 15), &empty_context(), "content");
        assert!(audit.findings.iter().any(|f| f.contains("No dependencies mapped")));
    }

    #[test]
    fn test_dependency_agent_high_fusion_risk() {
        let agent = DependencyAgent;
        let mut context = empty_context();
        for i in 0..5 {
            context.fusion_opportunities.push(format!("similar-skill-{}", i));
        }
        let audit = agent.assess(&test_metrics(1000, 5, 10, 15), &context, "content");
        assert!(audit.findings.iter().any(|f| f.contains("High fusion risk")));
    }

    // ─── LearningValueAgent ───────────────────────────────────────────────────

    #[test]
    fn test_learning_value_agent_no_gaps() {
        let agent = LearningValueAgent;
        let audit = agent.assess(&test_metrics(3000, 10, 15, 20), &empty_context(), "content");
        assert!(audit.findings.iter().any(|f| f.contains("well-rounded")));
        assert!(audit.score > 50.0, "No gaps should score well, got {}", audit.score);
    }

    #[test]
    fn test_learning_value_agent_many_gaps() {
        let agent = LearningValueAgent;
        let mut context = empty_context();
        for i in 0..5 {
            context.gaps.push(format!("Gap number {}", i));
        }
        let audit = agent.assess(&test_metrics(100, 0, 0, 0), &context, "content");
        assert!(audit.findings.iter().any(|f| f.contains("prioritize addressing")));
        assert!(audit.score < 50.0, "Many gaps should score low, got {}", audit.score);
    }

    #[test]
    fn test_learning_value_agent_improvement_history() {
        let agent = LearningValueAgent;
        let mut context = empty_context();
        context.improvement_history.push(crate::ImprovementRecord {
            date: "2026-01-01".to_string(),
            action: "Added examples".to_string(),
            impact: "Improved clarity".to_string(),
            evidence: "Metrics improved".to_string(),
        });
        let audit = agent.assess(&test_metrics(500, 2, 5, 5), &context, "content");
        assert!(audit.findings.iter().any(|f| f.contains("actively maintained")));
    }

    // ─── Orchestrated Audit ───────────────────────────────────────────────────

    #[test]
    fn test_run_orchestrated_audit() {
        let skill = crate::Skill {
            id: "orchestration-test".to_string(),
            label: "Orchestration Test".to_string(),
            description: "A comprehensive test skill with code, steps, and depth.".to_string(),
            experience: 60.0,
            level: "Epic".to_string(),
            confidence: 0.8,
            metrics: test_metrics(2000, 5, 10, 15),
            context: {
                let mut ctx = empty_context();
                ctx.dependencies.push(crate::SkillDep {
                    name: "rust".to_string(),
                    relationship: "uses".to_string(),
                    context: "core".to_string(),
                });
                ctx
            },
        };
        let content = "## Intro\n\nSome text.\n\n```python\nprint('hello')\n```\n";

        let audit = run_orchestrated_audit(&skill, content);
        assert_eq!(audit.skill_id, "orchestration-test");
        assert!(!audit.sub_audits.is_empty());
        assert_eq!(audit.sub_audits.len(), 4);

        // All sub-audits should have valid scores
        for sa in &audit.sub_audits {
            assert!(sa.score >= 0.0 && sa.score <= 100.0,
                "Sub-audit '{}' score {} out of range", sa.agent_name, sa.score);
            assert_eq!(sa.max_score, 100.0);
        }

        assert!(audit.overall_score >= 0.0 && audit.overall_score <= 100.0);
    }

    #[test]
    fn test_run_orchestrated_audit_all() {
        let skills = vec![
            crate::Skill {
                id: "skill-1".to_string(),
                label: "Skill 1".to_string(),
                description: "First skill".to_string(),
                experience: 50.0,
                level: "Epic".to_string(),
                confidence: 0.8,
                metrics: test_metrics(1000, 3, 5, 8),
                context: empty_context(),
            },
            crate::Skill {
                id: "skill-2".to_string(),
                label: "Skill 2".to_string(),
                description: "Second skill".to_string(),
                experience: 30.0,
                level: "Common".to_string(),
                confidence: 0.6,
                metrics: test_metrics(500, 1, 2, 3),
                context: empty_context(),
            },
        ];
        let mut contents = std::collections::HashMap::new();
        contents.insert("skill-1".to_string(), "Content for skill 1".to_string());
        contents.insert("skill-2".to_string(), "Content for skill 2".to_string());

        let audits = run_orchestrated_audit_all(&skills, &contents);
        assert_eq!(audits.len(), 2);
        assert_eq!(audits[0].sub_audits.len(), 4);
        assert_eq!(audits[1].sub_audits.len(), 4);
    }

    #[test]
    fn test_print_orchestrated_audit_does_not_panic() {
        let audit = OrchestratedAudit {
            skill_id: "test".to_string(),
            overall_score: 75.0,
            sub_audits: vec![
                SubAudit {
                    agent_name: "Doc".to_string(),
                    score: 80.0,
                    max_score: 100.0,
                    findings: vec!["Good".to_string()],
                    details: Default::default(),
                },
                SubAudit {
                    agent_name: "Code".to_string(),
                    score: 70.0,
                    max_score: 100.0,
                    findings: vec![],
                    details: Default::default(),
                },
            ],
            synthesized_summary: vec!["Overall: Good (75/100)".to_string()],
        };
        // Just verify it doesn't panic
        print_orchestrated_audit(&audit);
    }

    // ─── BackgroundSkillUpgradeAgent ─────────────────────────────────────────

    #[test]
    fn test_upgrade_agent_new() {
        let agent = BackgroundSkillUpgradeAgent::new();
        assert!(agent.upgrade_suggestions.is_empty());
        assert!(agent.last_run.is_none());
        assert_eq!(agent.skills_upgraded, 0);
    }

    #[test]
    fn test_upgrade_agent_default() {
        let agent = BackgroundSkillUpgradeAgent::default();
        assert!(agent.upgrade_suggestions.is_empty());
    }

    #[test]
    fn test_upgrade_agent_code_examples_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "test-skill".to_string(),
            label: "Test".to_string(),
            description: "A test skill".to_string(),
            experience: 30.0,
            level: "Common".to_string(),
            confidence: 0.5,
            metrics: crate::SkillMetrics {
                content_length: 500,
                code_blocks: 0,
                step_count: 8,
                tech_term_count: 10,
                complexity_score: 30.0,
                value_score: 40.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.category == "code_examples"));
        assert!(suggestions.iter().any(|s| s.suggestion.contains("code examples")));
    }

    #[test]
    fn test_upgrade_agent_content_length_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "short-skill".to_string(),
            label: "Short".to_string(),
            description: "Short skill".to_string(),
            experience: 10.0,
            level: "Poor".to_string(),
            confidence: 0.4,
            metrics: crate::SkillMetrics {
                content_length: 500,
                code_blocks: 3,
                step_count: 10,
                tech_term_count: 12,
                complexity_score: 50.0,
                value_score: 50.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        assert!(suggestions.iter().any(|s| s.category == "content_length"));
        // 3000 - 500 = 2500 chars needed → 2500/100 = 25 pts * 0.6 weight
        let content_sugg = suggestions.iter().find(|s| s.category == "content_length").unwrap();
        assert!((content_sugg.impact - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_upgrade_agent_step_instructions_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "no-steps".to_string(),
            label: "No Steps".to_string(),
            description: "Skill without steps".to_string(),
            experience: 20.0,
            level: "Common".to_string(),
            confidence: 0.5,
            metrics: crate::SkillMetrics {
                content_length: 3000,
                code_blocks: 8,
                step_count: 2,
                tech_term_count: 15,
                complexity_score: 70.0,
                value_score: 80.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        let step_sugg = suggestions.iter().find(|s| s.category == "step_instructions");
        assert!(step_sugg.is_some());
        let step_sugg = step_sugg.unwrap();
        // (10 - 2) * 2.0 * 0.6 = 9.6
        assert!((step_sugg.impact - 9.6).abs() < 0.1);
    }

    #[test]
    fn test_upgrade_agent_tech_terms_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "low-tech".to_string(),
            label: "Low Tech".to_string(),
            description: "Low tech skill".to_string(),
            experience: 20.0,
            level: "Common".to_string(),
            confidence: 0.5,
            metrics: crate::SkillMetrics {
                content_length: 3000,
                code_blocks: 8,
                step_count: 12,
                tech_term_count: 3,
                complexity_score: 60.0,
                value_score: 70.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        let tech_sugg = suggestions.iter().find(|s| s.category == "tech_terms");
        assert!(tech_sugg.is_some());
    }

    #[test]
    fn test_upgrade_agent_language_diversity_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let content = "\
```python
x = 1
```

More text

```python
y = 2
```";
        let skill = crate::Skill {
            id: "single-lang".to_string(),
            label: "Single Lang".to_string(),
            description: "Single language skill".to_string(),
            experience: 40.0,
            level: "Mythic".to_string(),
            confidence: 0.7,
            metrics: crate::SkillMetrics {
                content_length: 4000,
                code_blocks: 3,
                step_count: 12,
                tech_term_count: 15,
                complexity_score: 80.0,
                value_score: 85.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, content);
        assert!(suggestions.iter().any(|s| s.category == "language_diversity"));
    }

    #[test]
    fn test_upgrade_agent_gap_resolution_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let mut context = crate::SkillContext::default();
        context.gaps.push("Add more code examples (each block = +5 complexity)".to_string());
        context.gaps.push("Add step-by-step instructions (need 5+ for bonus)".to_string());

        let skill = crate::Skill {
            id: "gapped-skill".to_string(),
            label: "Gapped".to_string(),
            description: "A skill with gaps".to_string(),
            experience: 20.0,
            level: "Common".to_string(),
            confidence: 0.5,
            metrics: crate::SkillMetrics {
                content_length: 1000,
                code_blocks: 0,
                step_count: 2,
                tech_term_count: 5,
                complexity_score: 15.0,
                value_score: 30.0,
            },
            context,
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        let gap_suggs: Vec<_> = suggestions.iter().filter(|s| s.category == "gap_resolution").collect();
        assert!(!gap_suggs.is_empty());
    }

    #[test]
    fn test_upgrade_agent_deduplicates() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "dedup-skill".to_string(),
            label: "Dedup".to_string(),
            description: "A skill".to_string(),
            experience: 10.0,
            level: "Poor".to_string(),
            confidence: 0.3,
            metrics: crate::SkillMetrics {
                content_length: 100,
                code_blocks: 0,
                step_count: 0,
                tech_term_count: 0,
                complexity_score: 1.0,
                value_score: 50.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        let categories: Vec<&String> = suggestions.iter().map(|s| &s.category).collect();
        // Each category should appear at most once
        let unique_categories: std::collections::HashSet<_> = categories.iter().collect();
        assert_eq!(categories.len(), unique_categories.len());
    }

    #[test]
    fn test_upgrade_agent_sorts_by_priority() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "priority-test".to_string(),
            label: "Priority Test".to_string(),
            description: "Test".to_string(),
            experience: 10.0,
            level: "Poor".to_string(),
            confidence: 0.3,
            metrics: crate::SkillMetrics {
                content_length: 100,
                code_blocks: 0,
                step_count: 0,
                tech_term_count: 0,
                complexity_score: 1.0,
                value_score: 50.0,
            },
            context: crate::SkillContext::default(),
        };

        let suggestions = agent.analyze_for_upgrades(&skill, "");
        // First suggestion should have highest priority
        if suggestions.len() >= 2 {
            assert!(suggestions[0].priority >= suggestions[1].priority);
        }
    }

    #[test]
    fn test_generate_upgrade_plan() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "plan-test".to_string(),
            label: "Plan Test".to_string(),
            description: "Test skill".to_string(),
            experience: 30.0,
            level: "Common".to_string(),
            confidence: 0.5,
            metrics: crate::SkillMetrics {
                content_length: 500,
                code_blocks: 0,
                step_count: 8,
                tech_term_count: 10,
                complexity_score: 30.0,
                value_score: 40.0,
            },
            context: crate::SkillContext::default(),
        };

        agent.analyze_for_upgrades(&skill, "");
        let plan = agent.generate_upgrade_plan("plan-test");
        assert!(!plan.is_empty());

        let empty_plan = agent.generate_upgrade_plan("nonexistent-skill");
        assert!(empty_plan.is_empty());
    }

    #[test]
    fn test_apply_suggestion() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill = crate::Skill {
            id: "apply-test".to_string(),
            label: "Apply Test".to_string(),
            description: "Test".to_string(),
            experience: 30.0,
            level: "Common".to_string(),
            confidence: 0.5,
            metrics: crate::SkillMetrics {
                content_length: 500,
                code_blocks: 0,
                step_count: 8,
                tech_term_count: 10,
                complexity_score: 30.0,
                value_score: 40.0,
            },
            context: crate::SkillContext::default(),
        };
        let suggestion = UpgradeSuggestion {
            skill_id: "apply-test".to_string(),
            suggestion: "Add more code examples".to_string(),
            impact: 15.0,
            confidence: 0.8,
            category: "code_examples".to_string(),
            priority: 0.9,
        };

        let record = agent.apply_suggestion(&skill, &suggestion);
        assert_eq!(agent.skills_upgraded, 1);
        assert!(record.action.contains("Applied upgrade"));
        assert!(record.impact.contains("15"));
        assert!(record.evidence.contains("code_examples"));
    }

    #[test]
    fn test_get_summary() {
        let mut agent = BackgroundSkillUpgradeAgent::new();
        let skill1 = crate::Skill {
            id: "s1".to_string(),
            label: "S1".to_string(),
            description: "S1".to_string(),
            experience: 10.0,
            level: "Poor".to_string(),
            confidence: 0.3,
            metrics: crate::SkillMetrics {
                content_length: 100,
                code_blocks: 0,
                step_count: 0,
                tech_term_count: 0,
                complexity_score: 1.0,
                value_score: 50.0,
            },
            context: crate::SkillContext::default(),
        };
        let skill2 = crate::Skill {
            id: "s2".to_string(),
            label: "S2".to_string(),
            description: "S2".to_string(),
            experience: 10.0,
            level: "Poor".to_string(),
            confidence: 0.3,
            metrics: crate::SkillMetrics {
                content_length: 200,
                code_blocks: 1,
                step_count: 2,
                tech_term_count: 2,
                complexity_score: 5.0,
                value_score: 55.0,
            },
            context: crate::SkillContext::default(),
        };

        agent.analyze_for_upgrades(&skill1, "");
        agent.analyze_for_upgrades(&skill2, "");

        let (skills, total) = agent.get_summary();
        assert_eq!(skills, 2);
        assert!(total >= 2);
    }
}
