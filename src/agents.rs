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
        } else if metrics.content_length > 5000 {
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
