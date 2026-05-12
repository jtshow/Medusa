//! Medusa - Ultra-Fast Skill Scanner v0.12 (MSF)
//! Features: Audit-based ranking (60/30/10), auto-promotion, 9-tier system, context building, dreaming

mod dream;
mod outcomes;
mod agents;
mod procedural;

use std::path::Path;
use std::fs;
use std::time::Instant;
use std::collections::HashMap;
use walkdir::WalkDir;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Regex for YAML frontmatter fields.
    // FIXED: (?m) makes ^ match each line
    static ref RE_NAME: Regex = Regex::new(r"(?m)^name:\s*(.+)").unwrap();
    static ref RE_ID: Regex = Regex::new(r"(?m)^id:\s*(.+)").unwrap();
    static ref RE_DESC: Regex = Regex::new(r"(?m)^description:\s*(.+)").unwrap();

    // Regex for complexity analysis.
    static ref RE_CODE_BLOCK: Regex = Regex::new(r#"```[\s\S]*?```"#).unwrap();
    // FIXED: added (?m) multiline flag so ^ matches start of each line
    static ref RE_STEPS: Regex = Regex::new(r"(?m)^\s*(\d+\.|[-*])\s").unwrap();
    // FIXED: added (?i) case-insensitive flag
    static ref RE_TECH_TERMS: Regex = Regex::new(
        r"(?i)(algorithm|implementation|architecture|framework|optimization|scalability|security|encryption|authentication|database|api|sdk|middleware)"
    ).unwrap();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillMetrics {
    pub content_length: usize,
    pub code_blocks: usize,
    pub step_count: usize,
    pub tech_term_count: usize,
    pub complexity_score: f64,
    pub value_score: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub id: String,
    pub label: String,
    pub description: String,
    pub experience: f64,
    pub level: String,
    pub confidence: f64,
    pub metrics: SkillMetrics,
    pub context: SkillContext,  // NEW: Context information!
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct SkillContext {
    pub dependencies: Vec<SkillDep>,
    pub fusion_opportunities: Vec<String>,
    pub improvement_history: Vec<ImprovementRecord>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillDep {
    pub name: String,
    pub relationship: String,
    pub context: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImprovementRecord {
    pub date: String,
    pub action: String,
    pub impact: String,
    pub evidence: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ScanResult {
    skills: Vec<Skill>,
    total: usize,
    scan_time_ms: u64,
    fusion_matches: Vec<FusionMatch>,
    rust_used: bool,
    version: String,
    scan_type: String,
    learning_paths: Vec<LearningPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dream_summary: Option<DreamSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skill_outcomes: Option<Vec<SkillOutcome>>,
    #[serde(skip)]
    contents: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DreamSummary {
    patterns_found: usize,
    sessions_analyzed: usize,
    last_dream: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SkillOutcome {
    skill_id: String,
    level: String,
    score: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SharedMemoryBundle {
    source: String,
    exported_at: String,
    dreaming: dream::DreamKnowledgeBase,
    procedural: procedural::ProceduralMemory,
    outcomes: outcomes::OutcomeStore,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FusionMatch {
    skill1: String,
    skill2: String,
    similarity: f64,
    match_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MedusaConfig {
    #[serde(default = "default_complexity_weight")]
    complexity_weight: f64,
    #[serde(default = "default_value_weight")]
    value_weight: f64,
    #[serde(default = "default_keyword_weight")]
    keyword_weight: f64,
    #[serde(default)]
    tier_thresholds: HashMap<String, f64>,
    #[serde(default)]
    dreaming: DreamingConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DreamingConfig {
    #[serde(default = "default_dream_frequency")]
    frequency_scans: usize,
    #[serde(default = "default_dream_retention")]
    retention_percent: f64,
    #[serde(default = "default_dream_auto_apply")]
    auto_apply: bool,
    #[serde(default = "default_dream_max_insights")]
    max_insights: usize,
}

impl Default for DreamingConfig {
    fn default() -> Self {
        DreamingConfig {
            frequency_scans: default_dream_frequency(),
            retention_percent: default_dream_retention(),
            auto_apply: default_dream_auto_apply(),
            max_insights: default_dream_max_insights(),
        }
    }
}

impl Default for MedusaConfig {
    fn default() -> Self {
        MedusaConfig {
            complexity_weight: 0.6,
            value_weight: 0.3,
            keyword_weight: 0.1,
            tier_thresholds: HashMap::new(),
            dreaming: DreamingConfig::default(),
        }
    }
}

fn default_complexity_weight() -> f64 { 0.6 }
fn default_value_weight() -> f64 { 0.3 }
fn default_keyword_weight() -> f64 { 0.1 }
fn default_dream_frequency() -> usize { 1 }
fn default_dream_retention() -> f64 { 0.8 }
fn default_dream_auto_apply() -> bool { true }
fn default_dream_max_insights() -> usize { 200 }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct ScanCache {
    entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    hash: u64,
    skill: Skill,
}

#[derive(Debug, Clone, serde::Serialize)]
struct LearningPath {
    name: String,
    description: String,
    skills: Vec<String>,
    total_experience: f64,
}

/// Analyze skill complexity (60% weight in scoring!)
fn analyze_skill_complexity(content: &str) -> SkillMetrics {
    let content_length = content.len();
    
    // Count code blocks (```...```) - 25 points max.
    let code_blocks = RE_CODE_BLOCK.find_iter(content).count();
    
    // Count step-by-step instructions - 20 points max.
    let step_count = RE_STEPS.find_iter(content).count();
    
    // Count technical terms - 25 points max.
    let tech_term_count = RE_TECH_TERMS.find_iter(&content.to_lowercase()).count();
    
    // Calculate complexity score (0-100) - THIS IS 60% OF TOTAL!
    let mut complexity = 0.0_f64;
    
    // Length factor (max 30 points).
    complexity += (content_length as f64 / 100.0).min(30.0);
    
    // Code blocks factor (max 25 points) - BIGGEST LEVER!
    complexity += (code_blocks as f64 * 5.0).min(25.0);
    
    // Steps factor (max 20 points).
    complexity += (step_count as f64 * 2.0).min(20.0);
    
    // Technical terms factor (max 25 points).
    complexity += (tech_term_count as f64 * 2.5).min(25.0);
    
    // Bonus for having all components (max 10 points).
    if code_blocks > 0 && step_count > 5 && tech_term_count > 3 {
        complexity += 10.0;
    }
    
    complexity = complexity.min(100.0);
    
    // Calculate value score (30% weight in scoring).
    let mut value: f64 = 50.0; // Base value.
    
    if content_length > 500 { value += 10.0; }
    if code_blocks > 0 { value += 15.0; }
    if step_count > 10 { value += 10.0; }
    if tech_term_count > 5 { value += 15.0; }
    
    value = value.min(100.0);
    
    SkillMetrics {
        content_length,
        code_blocks,
        step_count,
        tech_term_count,
        complexity_score: complexity,
        value_score: value,
    }
}

/// Extract YAML frontmatter.
fn extract_frontmatter_str(content: &str) -> Option<&str> {
    if !content.starts_with("---") || content.len() < 5 {
        return None;
    }
    let start = 4;
    content[start..].find("\n---").map(|pos| &content[start..start + pos])
}

/// Parse YAML field with regex.
fn parse_field_regex(re: &Regex, fm: &str) -> Option<String> {
    re.captures(fm).map(|cap| cap[1].to_string())
}

/// Parse SKILL.md with FULL context building.
fn parse_skill_md(content: &str, file_path: &Path, config: &MedusaConfig) -> Option<Skill> {
    let fm = extract_frontmatter_str(content)?;

    let id = parse_field_regex(&RE_ID, fm)
        .or_else(|| parse_field_regex(&RE_NAME, fm))
        .unwrap_or_else(|| {
            file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string()
        });

    let label = parse_field_regex(&RE_NAME, fm).unwrap_or_else(|| id.clone());
    let description = parse_field_regex(&RE_DESC, fm).unwrap_or_default();
    
    // FULL audit-based analysis.
    let metrics = analyze_skill_complexity(content);
    let experience = calculate_experience(&metrics, &description, config);
    let level = get_level(experience);
    let confidence = calculate_confidence(&description, &label, &metrics);
    
    // Build context automatically.
    let context = build_skill_context(&id, &metrics, content);
    
    Some(Skill {
        id,
        label,
        description,
        experience,
        level,
        confidence,
        metrics,
        context,
    })
}

/// Export skills to CSV format
fn export_csv(skills: &[Skill], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = String::new();
    csv.push_str("ID,Label,Description,Experience,Level,Confidence,ContentLength,CodeBlocks,Steps,TechTerms,Complexity,Value\n");
    
    for skill in skills {
        let desc_escaped = skill.description.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},\"{}\",\"{}\",{:.1},{},{:.2},{},{},{},{},{:.1},{:.1}\n",
            skill.id, skill.label, desc_escaped,
            skill.experience, skill.level, skill.confidence,
            skill.metrics.content_length, skill.metrics.code_blocks,
            skill.metrics.step_count, skill.metrics.tech_term_count,
            skill.metrics.complexity_score, skill.metrics.value_score
        ));
    }
    
    fs::write(output_path, csv)?;
    Ok(())
}

/// Export skills to Markdown format
fn export_markdown(skills: &[Skill], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut md = String::new();
    md.push_str("# Medusa Skill Report\n\n");
    md.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    md.push_str("## Skills (Sorted by Experience)\n\n");
    
    for skill in skills {
        md.push_str(&format!("### {} ({})\n", skill.label, skill.level));
        md.push_str(&format!("- **Experience**: {:.1}/100\n", skill.experience));
        md.push_str(&format!("- **Confidence**: {:.0}%\n", skill.confidence * 100.0));
        md.push_str(&format!("- **Description**: {}\n\n", skill.description));
        
        md.push_str("**Metrics:**\n");
        md.push_str(&format!("- Content: {} chars\n", skill.metrics.content_length));
        md.push_str(&format!("- Code Blocks: {}\n", skill.metrics.code_blocks));
        md.push_str(&format!("- Steps: {}\n", skill.metrics.step_count));
        md.push_str(&format!("- Tech Terms: {}\n\n", skill.metrics.tech_term_count));
        
        if !skill.context.gaps.is_empty() {
            md.push_str("**Gaps:**\n");
            for gap in &skill.context.gaps {
                md.push_str(&format!("- {}\n", gap));
            }
            md.push('\n');
        }
        md.push_str("---\n\n");
    }
    
    fs::write(output_path, md)?;
    Ok(())
}

/// Export skills to SVG visualization
fn export_svg(skills: &[Skill], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut svg = String::new();
    let width = 1200;
    let height = 100 + skills.len() * 60;
    
    svg.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\n", width, height));
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#0a0e27\"/>\n");
    svg.push_str("<text x=\"20\" y=\"40\" fill=\"#00ff41\" font-family=\"monospace\" font-size=\"24\">Medusa Skill Visualization</text>\n");
    
    for (i, skill) in skills.iter().enumerate() {
        let y = 80 + i * 60;
        let bar_width = (skill.experience * 10.0) as u32;
        
        let color = match skill.level.as_str() {
            "Godlike" => "#ff6600",
            "Unique" => "#ff0000",
            "Legendary" => "#ff00ff",
            "Mythic" => "#9900ff",
            "Epic" => "#ffcc00",
            "Ultra Rare" => "#00aa88",
            "Rare" => "#0088ff",
            "Uncommon" => "#00ff41",
            "Common" => "#cccccc",
            _ => "#333333",
        };
        
        svg.push_str(&format!(
            "<rect x=\"20\" y=\"{}\" width=\"{}\" height=\"40\" fill=\"{}\" rx=\"5\"/>\n",
            y, bar_width, color
        ));
        svg.push_str(&format!(
            "<text x=\"30\" y=\"{}\" fill=\"white\" font-family=\"monospace\" font-size=\"14\">{} ({}) - {:.1}</text>\n",
            y + 25, skill.label, skill.level, skill.experience
        ));
    }
    
    svg.push_str("</svg>");
    fs::write(output_path, svg)?;
    Ok(())
}

/// Load Medusa configuration from medusa.toml
fn load_config(path: &Path) -> MedusaConfig {
    let config_path = path.join("medusa.toml");
    if config_path.exists() {
        fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| {
                let filtered: String = s.lines()
                    .filter(|l| !l.trim_start().starts_with('#'))
                    .collect::<Vec<_>>()
                    .join("\n");
                toml::from_str(&filtered).ok()
            })
            .unwrap_or_default()
    } else {
        MedusaConfig::default()
    }
}

/// Calculate experience using configurable weights
fn calculate_experience(metrics: &SkillMetrics, description: &str, config: &MedusaConfig) -> f64 {
    let mut exp = 10.0;
    
    // Complexity-based scoring (configurable weight)
    exp += metrics.complexity_score * config.complexity_weight;
    
    // Value-based scoring (configurable weight)
    exp += metrics.value_score * config.value_weight;
    
    // Description keyword bonuses (remaining weight)
    let desc_lower = description.to_lowercase();
    let keyword_bonuses = [
        ("advanced", 5.0), ("expert", 8.0), ("senior", 6.0),
        ("react", 3.0), ("vue", 3.0), ("angular", 3.0),
        ("security", 5.0), ("owasp", 6.0), ("penetration", 6.0),
        ("rust", 5.0), ("python", 3.0), ("javascript", 2.0),
        ("kubernetes", 6.0), ("docker", 4.0), ("aws", 5.0),
        ("machine learning", 8.0), ("ai", 5.0), ("llm", 6.0),
    ];
    
    for (kw, score) in keyword_bonuses {
        if desc_lower.contains(kw) {
            exp += score * config.keyword_weight;
        }
    }
    // Clamp to avoid floating point drift above 100
    exp = exp.min(100.0);
    (exp * 100.0).round() / 100.0
}

/// Build learning paths from skills
fn build_learning_paths(skills: &[Skill]) -> Vec<LearningPath> {
    let mut paths = Vec::new();
    
    // Group by category (extract from skill id)
    let mut categories: HashMap<String, Vec<&Skill>> = HashMap::new();
    for skill in skills {
        let category = skill.id.split('-').next().unwrap_or("other").to_string();
        categories.entry(category).or_default().push(skill);
    }
    
    for (category, category_skills) in categories {
        if category_skills.len() < 2 {
            continue;
        }
        
        let mut sorted_skills = category_skills.clone();
        sorted_skills.sort_by(|a, b| a.experience.partial_cmp(&b.experience).unwrap_or(std::cmp::Ordering::Equal));
        
        let skill_names: Vec<String> = sorted_skills.iter().map(|s| s.id.clone()).collect();
        let total_exp: f64 = sorted_skills.iter().map(|s| s.experience).sum();
        
        paths.push(LearningPath {
            name: format!("{} Learning Path", category.to_uppercase()),
            description: format!("Master {} skills from beginner to expert", category),
            skills: skill_names,
            total_experience: total_exp,
        });
    }
    
    paths
}

/// Get 9-tier level (based on experience score).
fn get_level(exp: f64) -> String {
    match exp {
        e if e >= 95.0 => "Godlike",      // 95+ (Red-Orange-Green gradient)
        e if e >= 90.0 => "Unique",        // 90+ (Red)
        e if e >= 85.0 => "Legendary",     // 85+ (Pink-Purple)
        e if e >= 80.0 => "Mythic",        // 80+ (Purple)
        e if e >= 75.0 => "Epic",          // 75+ (Yellow)
        e if e >= 65.0 => "Ultra Rare",    // 65+ (Teal)
        e if e >= 55.0 => "Rare",          // 55+ (Blue)
        e if e >= 45.0 => "Uncommon",      // 45+ (Green)
        e if e >= 25.0 => "Common",        // 25+ (Light Gray)
        _ => "Poor",                    // <25 (Dark Gray)
    }.to_string()
}

/// Build context around a skill (Automatic context generation!)
fn build_skill_context(skill_id: &str, metrics: &SkillMetrics, content: &str) -> SkillContext {
    let mut context = SkillContext::default();
    
    // Identify gaps based on Medusa's algorithm
    if metrics.step_count < 5 {
        context.gaps.push("Add step-by-step instructions (need 5+ for bonus)".to_string());
    }
    if metrics.code_blocks < 10 {
        context.gaps.push("Add more code examples (each block = +5 complexity)".to_string());
    }
    if metrics.tech_term_count < 12 {
        context.gaps.push("Add technical terms (each = +2.5 complexity)".to_string());
    }
    if metrics.content_length < 3000 {
        context.gaps.push("Expand content (need 3000+ chars)".to_string());
    }
    
    // Map dependencies (simplified - would need SKILL.md parsing).
    match skill_id {
        "ai-ml" => {
            context.dependencies.push(SkillDep {
                name: "agent-framework-azure-ai-py".to_string(),
                relationship: "uses Azure AI Foundry".to_string(),
                context: "Both deal with AI agent architectures".to_string(),
            });
            context.dependencies.push(SkillDep {
                name: "agent-memory-mcp".to_string(),
                relationship: "memory systems".to_string(),
                context: "AI/ML workflows need memory".to_string(),
            });
        }
        "active-directory-attacks" => {
            context.dependencies.push(SkillDep {
                name: "security-advanced".to_string(),
                relationship: "security ecosystem".to_string(),
                context: "Part of offensive security toolset".to_string(),
            });
        }
        _ => {}
    }
    
    // Detect real dependencies from content
    let content_lower = content.to_lowercase();
    let common_skills = ["rust", "python", "javascript", "react", "docker", "kubernetes", "aws", "azure", "gcp"];
    for skill in common_skills.iter() {
        if content_lower.contains(skill) && !skill_id.contains(skill) {
            context.dependencies.push(SkillDep {
                name: skill.to_string(),
                relationship: "references".to_string(),
                context: format!("Content mentions {}", skill),
            });
        }
    }
    
    context
}

/// Calculate confidence.
fn calculate_confidence(_description: &str, _label: &str, metrics: &SkillMetrics) -> f64 {
    let mut conf: f64 = 0.3;
    
    if metrics.content_length > 100 { conf += 0.2; }
    if metrics.content_length > 300 { conf += 0.15; }
    
    if metrics.code_blocks > 0 { conf += 0.15; }
    if metrics.step_count > 5 { conf += 0.1; }
    if metrics.tech_term_count > 3 { conf += 0.1; }
    
    conf.min(1.0)
}

/// Scan skills with FULL audit and context building.
fn scan_skills(path: &str, parallel: bool, use_cache: bool) -> Result<ScanResult, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let path = Path::new(path);
    let config = load_config(path);

    if !path.is_dir() {
        return Ok(ScanResult {
            skills: vec![],
            total: 0,
            scan_time_ms: 0,
            fusion_matches: vec![],
            learning_paths: vec![],
            dream_summary: None,
            skill_outcomes: None,
            rust_used: true,
            version: "0.12.1".to_string(),
            scan_type: if parallel { "parallel" } else { "sequential" }.to_string(),
            contents: HashMap::new(),
        });
    }

    // Load cache if enabled
    let cache_path = path.join(".medusa_cache.json");
    let mut cache: ScanCache = if use_cache && cache_path.exists() {
        fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        ScanCache::default()
    };

    let entries: Vec<_> = WalkDir::new(path)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().file_name().is_some_and(|n| n == "SKILL.md"))
        .collect();

    let mut contents: HashMap<String, String> = HashMap::new();
    let skills: Vec<_> = if parallel {
        let mut new_skills = Vec::new();
        let mut cache_updates: Vec<(String, u64, Skill)> = Vec::new();
        
        for entry in entries {
            let path_str = entry.path().to_string_lossy().to_string();
            
            // Calculate content hash for better caching
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let hash = fxhash::hash64(&content.as_bytes());
                
                // Check cache
                if use_cache {
                    if let Some(cached) = cache.entries.get(&path_str) {
                        if cached.hash == hash {
                            contents.insert(cached.skill.id.clone(), content);
                            new_skills.push(cached.skill.clone());
                            continue;
                        }
                    }
                }
                
                if let Some(skill) = parse_skill_md(&content, entry.path(), &config) {
                    contents.insert(skill.id.clone(), content);
                    cache_updates.push((path_str.clone(), hash, skill));
                }
            }
        }
        
        // Update cache
        for (path_str, hash, skill) in cache_updates {
            cache.entries.insert(path_str, CacheEntry { hash, skill: skill.clone() });
            new_skills.push(skill);
        }
        
        new_skills
    } else {
        entries
            .iter()
            .filter_map(|entry| {
                let content = fs::read_to_string(entry.path()).ok()?;
                if let Some(skill) = parse_skill_md(&content, entry.path(), &config) {
                    contents.insert(skill.id.clone(), content);
                    Some(skill)
                } else {
                    None
                }
            })
            .collect()
    };

    // Save cache (skip if serialization fails to avoid corruption)
    if use_cache {
        if let Ok(json) = serde_json::to_string_pretty(&cache) {
            let _ = fs::write(&cache_path, json);
        }
    }

    let mut skills = skills;
    skills.sort_by(|a, b| b.experience.partial_cmp(&a.experience).unwrap_or(std::cmp::Ordering::Equal));

    let fusion_matches = detect_fusion(&skills);
    let learning_paths = build_learning_paths(&skills);

    let elapsed = start.elapsed();

    // Cross-session dream context
    let dream_summary = {
        let kb = dream::load_knowledge_base_from_path(path);
        if kb.total_patterns_found > 0 {
            Some(DreamSummary {
                patterns_found: kb.total_patterns_found,
                sessions_analyzed: kb.total_sessions_analyzed,
                last_dream: kb.last_dream_time.unwrap_or_else(|| "never".to_string()),
            })
        } else {
            None
        }
    };

    // Outcome assessments
    let outcome_store = outcomes::load_outcomes(path);
    let skill_outcomes: Option<Vec<SkillOutcome>> = if !outcome_store.rubrics.is_empty() {
        Some(skills.iter().filter_map(|s| {
            outcomes::assess_skill(&s.id, s.metrics.content_length, s.metrics.code_blocks, s.metrics.step_count, s.metrics.tech_term_count, &outcome_store)
                .map(|a| SkillOutcome { skill_id: s.id.clone(), level: a.level, score: a.score })
        }).collect())
    } else {
        None
    };

    // Extract procedural workflows from skill content
    procedural::extract_workflows_from_skills(&skills, &contents, path);

    Ok(ScanResult {
        total: skills.len(),
        scan_time_ms: elapsed.as_millis() as u64,
        skills,
        contents,
        fusion_matches,
        learning_paths,
        dream_summary,
        skill_outcomes,
        rust_used: true,
        version: "0.12.1".to_string(),
        scan_type: if parallel { "parallel" } else { "sequential" }.to_string(),
    })
}

/// Detect fusion (similar skills).
fn detect_fusion(skills: &[Skill]) -> Vec<FusionMatch> {
    let mut matches = Vec::new();

    for (i, s1) in skills.iter().enumerate() {
        for s2 in skills.iter().skip(i + 1) {
            let name_sim = string_similarity(&s1.label, &s2.label);
            let desc_sim = if !s1.description.is_empty() && !s2.description.is_empty() {
                string_similarity(&s1.description, &s2.description)
            } else {
                0.0
            };

            let similarity = (name_sim * 0.6 + desc_sim * 0.4).min(1.0);

            if similarity > 0.5 {
                matches.push(FusionMatch {
                    skill1: s1.id.clone(),
                    skill2: s2.id.clone(),
                    similarity,
                    match_type: if name_sim > desc_sim { "name" } else { "content" }.to_string(),
                });
            }
        }
    }

    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(20);
    matches
}

/// String similarity using FxHash.
fn string_similarity(s1: &str, s2: &str) -> f64 {
    if s1.is_empty() || s2.is_empty() { return 0.0; }
    if s1 == s2 { return 1.0; }

    let words1: Vec<&str> = s1.split_whitespace().collect();
    let words2: Vec<&str> = s2.split_whitespace().collect();

    let mut common = 0;
    for w1 in &words1 {
        if words2.contains(w1) {
            common += 1;
        }
    }

    let total = words1.len() + words2.len();
    if total == 0 { return 0.0; }

    2.0 * common as f64 / total as f64
}

/// Print help.
fn print_help() {
    println!("Medusa Skill Framework (MSF) v0.12.0 - Audit-Based Ranking with Context");
    println!("Usage: medusa <command> [options]");
    println!("\nCommands:");
    println!("  scan <path>              Scan skills with FULL audit (60/30/10 scoring)");
    println!("    --sequential           Use sequential scanning (no Rayon)");
    println!("    --no-cache             Disable incremental scan cache");
    println!("\n  audit <path>            Show detailed skill audit with context");
    println!("    --no-cache             Disable incremental scan cache");
    println!("\n  html <path> <output>   Generate HTML visualization");
    println!("    --sequential           Use sequential scanning");
    println!("    --no-cache             Disable cache");
    println!("\n  export-csv <path> <f>  Export skills to CSV format");
    println!("  export-md <path> <f>   Export skills to Markdown format");
    println!("  export-svg <path> <f>  Export skills to SVG visualization");
    println!("\n  dream <path>            Run dreaming process (cross-session pattern detection)");
    println!("  dream-status <path>     Show dream knowledge base");
    println!("  dream-reset <path>      Reset dream state and history");
    println!("  dream-consolidate <path> Manually consolidate dream knowledge base");
    println!("  dream-diary <path>       Show dream diary (narrative skill evolution timeline)");
    println!("    --output <file.md>     Export diary as Markdown");
    println!("  dream-params <path>      Show dreaming configuration parameters");
    println!("\nProcedural Memory:");
    println!("  procedural-list <path>    List all learned procedural workflows");
    println!("  procedural-show <path> <id>  Show workflows associated with a skill");
    println!("\nMemory Sharing:");
    println!("  memory-export <path> <f>  Export all memory (dream, procedural, outcomes) to a JSON bundle");
    println!("  memory-import <path> <f>  Import and merge a memory bundle from another Medusa instance");
    println!("    --source <name>        Tag imported data with a source identifier (default: 'shared')");
    println!("\nOrchestration:");
    println!("  orchestrate <path>       Run multi-agent orchestrated audit (4 specialized sub-audits)");
    println!("    --sequential           Use sequential scanning");
    println!("    --no-cache             Disable cache");
    println!("\n  outcome-add <path> <id>  Add default outcome rubric for a skill");
    println!("  outcome-list <path>      List outcome rubrics");
    println!("  outcome-remove <path> <id>  Remove an outcome rubric");
    println!("  learning-path <path> <id>  Show learning path and suggestions for a skill");
    println!("\n  ab-test <path>          Run A/B test (parallel vs sequential)");
    println!("    --iterations N         Number of test iterations (default: 10)");
    println!("\n  update                  Update Medusa from GitHub (git pull + rebuild)");
    println!("\nOptions:");
    println!("  --help, -h              Show this help message");
    println!("  --version, -v           Show version");
    println!("\nExamples:");
    println!("  medusa scan /path/to/skills                    # JSON with context");
    println!("  medusa audit /path/to/skills/ai-ml         # Detailed audit");
    println!("  medusa html /path/to/skills report.html     # Visual report");
    println!("  medusa export-csv /path/to/skills skills.csv # CSV export");
}

/// Print audit report with FULL context and optional cross-session dream context.
fn print_audit_report(skills: &[Skill], dream_kb: Option<&dream::DreamKnowledgeBase>) {
    println!("\n=== Medusa Skill Audit Report (v0.12) ===\n");
    
    for skill in skills {
        println!("Skill: {} ({})", skill.label, skill.id);
        println!("  Level: {} (Experience: {:.1}/100)", skill.level, skill.experience);
        println!("  Confidence: {:.0}%", skill.confidence * 100.0);
        println!("\n  Metrics (60% weight in scoring):");
        println!("    - Content Length: {} chars ({} pts)", skill.metrics.content_length, 
            (skill.metrics.content_length as f64 / 100.0).min(30.0));
        println!("    - Code Blocks: {} ({} pts)", skill.metrics.code_blocks, 
            (skill.metrics.code_blocks as f64 * 5.0).min(25.0));
        println!("    - Step Instructions: {} ({} pts)", skill.metrics.step_count,
            (skill.metrics.step_count as f64 * 2.0).min(20.0));
        println!("    - Technical Terms: {} ({} pts)", skill.metrics.tech_term_count,
            (skill.metrics.tech_term_count as f64 * 2.5).min(25.0));
        println!("\n  Scores:");
        println!("    - Complexity Score: {:.1}/100 (60% weight)", skill.metrics.complexity_score);
        println!("    - Value Score: {:.1}/100 (30% weight)", skill.metrics.value_score);
        println!("\n  Context & Gaps:");
        for gap in &skill.context.gaps {
            println!("    - Gap: {}", gap);
        }
        if !skill.context.dependencies.is_empty() {
            println!("\n  Dependencies:");
            for dep in &skill.context.dependencies {
                println!("    - {}: {} ({})", dep.name, dep.relationship, dep.context);
            }
        }
        
        // Cross-session learning context (from dreaming process)
        if let Some(kb) = dream_kb {
            let cross_session = dream::get_cross_session_summary(kb, &skill.id);
            if !cross_session.is_empty() {
                println!("\n  Cross-Session Insights (from dreaming):");
                for line in &cross_session {
                    println!("    - {}", line);
                }
            }
        }
        println!();
    }
    
    // Summary.
    if skills.is_empty() {
        println!("=== Summary ===");
        println!("Total Skills: 0");
        return;
    }
    let avg_complexity = skills.iter().map(|s| s.metrics.complexity_score).sum::<f64>() / skills.len() as f64;
    let avg_value = skills.iter().map(|s| s.metrics.value_score).sum::<f64>() / skills.len() as f64;
    
    println!("=== Summary ===");
    println!("Total Skills: {}", skills.len());
    println!("Average Complexity: {:.1}/100 (60% of ranking)", avg_complexity);
    println!("Average Value: {:.1}/100 (30% of ranking)", avg_value);
    
    if let Some(kb) = dream_kb {
        if kb.total_patterns_found > 0 {
            println!("Dream Patterns Available: {} (run 'medusa dream-status <path>' for full report)", kb.total_patterns_found);
        }
    }
}

/// Generate HTML with context and cross-session learning.
fn generate_html(result: &ScanResult, output_path: &str, dream_kb: Option<&dream::DreamKnowledgeBase>) -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::new();
    
    html.push_str("        body { font-family: monospace; background: #0a0e27; color: #00ff41; margin: 20px; }\n");
    html.push_str("        h1 { color: #00ff41; text-shadow: 0 0 10px #00ff41; }\n");
    html.push_str("        .skill { background: #1a1f3a; border: 1px solid #00ff41; padding: 15px; margin: 10px 0; border-radius: 5px; }\n");
    html.push_str("        .level-godlike { border-left: 5px solid #ff6600; background: linear-gradient(90deg, #ff0000, #ff6600, #00ff41); }\n");
    html.push_str("        .level-unique { border-left: 5px solid #ff0000; background: #ff0000; }\n");
    html.push_str("        .level-legendary { border-left: 5px solid #ff00ff; background: #ff00ff; }\n");
    html.push_str("        .level-mythic { border-left: 5px solid #9900ff; background: #9900ff; }\n");
    html.push_str("        .level-epic { border-left: 5px solid #ffcc00; background: #ffcc00; }\n");
    html.push_str("        .level-ultra-rare { border-left: 5px solid #00aa88; background: #00aa88; }\n");
    html.push_str("        .level-rare { border-left: 5px solid #0088ff; background: #0088ff; }\n");
    html.push_str("        .level-uncommon { border-left: 5px solid #00ff41; background: #00ff41; }\n");
    html.push_str("        .level-common { border-left: 5px solid #cccccc; background: #cccccc; }\n");
    html.push_str("        .level-poor { border-left: 5px solid #333333; background: #333333; }\n");
    html.push_str("        .fusion { background: #2a1f3a; border: 1px solid #ff00ff; padding: 10px; margin: 5px 0; }\n");
    html.push_str("        .meta { color: #888; font-size: 12px; }\n");
    html.push_str("        .bar { background: #333; height: 20px; border-radius: 10px; overflow: hidden; }\n");
    html.push_str("        .bar-fill { background: linear-gradient(90deg, #00ff41, #00aaff); height: 100%; }\n");
    html.push_str("        .metrics { font-size: 11px; color: #aaa; margin-top: 5px; }\n");
    html.push_str("        .context { font-size: 10px; color: #888; margin-top: 3px; }\n");
    html.push_str("        .dream { font-size: 10px; color: #ff00ff; margin-top: 3px; }\n");
    html.push_str("        .dream-section { background: #1a0a2e; border: 1px solid #ff00ff; padding: 10px; margin: 10px 0; border-radius: 5px; }\n");
    html.push_str("    </style>\n</head>\n<body>\n");
    
    html.push_str(&format!("    <h1>Medusa Scan Report (v0.12)</h1>\n    <div class=\"meta\"><p>Total Skills: {} | Scan Time: {}ms | Version: {} | Type: {}</p></div>\n",
        result.total, result.scan_time_ms, result.version, result.scan_type));
    
    html.push_str("    <h2>Skills (Sorted by Experience)</h2>\n    <div id=\"skills\">\n");
    for s in &result.skills {
        let mut dream_line = String::new();
        if let Some(kb) = dream_kb {
            let cross = dream::get_cross_session_summary(kb, &s.id);
            if !cross.is_empty() {
                dream_line = format!("<p class=\"dream\">Cross-Session: {}</p>", cross.join(" | "));
            }
        }
        html.push_str(&format!(
            "        <div class=\"skill level-{}\"><h3>{} <span class=\"meta\">[{}]</span></h3><p>{}</p><div class=\"bar\"><div class=\"bar-fill\" style=\"width: {}%\"></div></div><p class=\"meta\">ID: {} | Exp: {} | Conf: {}%</p><p class=\"metrics\">Len: {} | Code: {} | Steps: {} | Terms: {} | Comp: {:.1} | Val: {:.1}</p><p class=\"context\">Gaps: {}</p>{}</div>\n",
            s.level.to_lowercase(), s.label, s.level, s.description, s.experience, s.id, s.experience, (s.confidence * 100.0).floor(),
            s.metrics.content_length, s.metrics.code_blocks, s.metrics.step_count, s.metrics.tech_term_count, s.metrics.complexity_score, s.metrics.value_score,
            s.context.gaps.join(", "), dream_line
        ));
    }
    html.push_str("    </div>\n");
    
    if let Some(kb) = dream_kb {
        if kb.total_patterns_found > 0 {
            html.push_str("    <div class=\"dream-section\">\n");
            html.push_str(&format!("    <h2>Dream Patterns ({} total)</h2>\n", kb.total_patterns_found));
            html.push_str(&format!("    <p class=\"meta\">Sessions Analyzed: {} | Last Dream: {}</p>\n",
                kb.total_sessions_analyzed, kb.last_dream_time.as_deref().unwrap_or("never")));
            html.push_str("    </div>\n");
        }
    }
    
    html.push_str("    <h2>Fusion Detection (Similar Skills)</h2>\n    <div id=\"fusion\">\n");
    for f in &result.fusion_matches {
        html.push_str(&format!(
            "        <div class=\"fusion\"><p><strong>{}</strong> ↔ <strong>{}</strong> ({}% similar, {})</p></div>\n",
            f.skill1, f.skill2, (f.similarity * 100.0).floor(), f.match_type
        ));
    }
    html.push_str("    </div>\n</body>\n</html>");
    
    fs::write(output_path, html)?;
    Ok(())
}

/// Run A/B test.
fn run_ab_test(path: &str, iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    if iterations == 0 {
        eprintln!("Error: --iterations must be at least 1");
        return Ok(());
    }
    eprintln!("Running A/B Test: Parallel vs Sequential Scan");
    eprintln!("Path: {}", path);
    eprintln!("Iterations: {}", iterations);
    eprintln!("\nHypothesis: Parallel scanning is faster than sequential");
    eprintln!("Primary metric: scan_time_ms\n");

    let mut parallel_times = Vec::new();
    let mut sequential_times = Vec::new();

    for i in 1..=iterations {
        let parallel_result = scan_skills(path, true, true)?;
        parallel_times.push(parallel_result.scan_time_ms);
        
        let sequential_result = scan_skills(path, false, true)?;
        sequential_times.push(sequential_result.scan_time_ms);

        eprintln!("Iteration {}: Parallel={}ms, Sequential={}ms",
            i, parallel_result.scan_time_ms, sequential_result.scan_time_ms);
    }

    let parallel_avg = parallel_times.iter().sum::<u64>() as f64 / iterations as f64;
    let sequential_avg = sequential_times.iter().sum::<u64>() as f64 / iterations as f64;

    eprintln!("\n=== A/B Test Results ===");
    eprintln!("Parallel avg: {:.2}ms", parallel_avg);
    eprintln!("Sequential avg: {:.2}ms", sequential_avg);

    if parallel_avg < sequential_avg {
        let improvement = (sequential_avg - parallel_avg) / sequential_avg * 100.0;
        eprintln!("✅ Parallel is {:.1}% faster", improvement);
    } else {
        eprintln!("❌ Parallel is not faster (may need more samples or different workload)");
    }

    Ok(())
}

/// Main entry point.
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "--version" | "-v" => {
            println!("Medusa Skill Framework (MSF) v0.12.0");
        }
        "scan" => {
            if args.len() < 3 {
                eprintln!("Error: Missing path argument");
                eprintln!("Usage: medusa scan <path> [--sequential] [--no-cache]");
                return;
            }
            let path = &args[2];
            let sequential = args.iter().any(|a| a == "--sequential");
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(path, !sequential, use_cache) {
                Ok(result) => {
                    let snapshots: Vec<dream::SkillSnapshot> = result.skills.iter().map(dream::from_skill).collect();
                    dream::record_session(Path::new(path), &snapshots);
                    match serde_json::to_string_pretty(&result) {
                        Ok(json) => println!("{}", json),
                        Err(e) => eprintln!("Error serializing scan output: {}. This may be caused by invalid numeric values (NaN) in skill scores.", e),
                    }
                }
                Err(e) => eprintln!("Scan failed: {}. Check that the path exists and contains SKILL.md files.", e),
            }
        }
        "html" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: medusa html <path> <output> [--sequential] [--no-cache]");
                return;
            }
            let path = &args[2];
            let output = &args[3];
            let sequential = args.iter().any(|a| a == "--sequential");
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(path, !sequential, use_cache) {
                Ok(result) => {
                    let snapshots: Vec<dream::SkillSnapshot> = result.skills.iter().map(dream::from_skill).collect();
                    dream::record_session(Path::new(path), &snapshots);
                    let dream_kb = Some(dream::load_knowledge_base_from_path(Path::new(path)));
                    match generate_html(&result, output, dream_kb.as_ref()) {
                        Ok(_) => eprintln!("HTML report generated: {}", output),
                        Err(e) => eprintln!("Error writing HTML report to '{}': {}. Check that the output path is writable.", output, e),
                    }
                }
                Err(e) => eprintln!("Scan failed: {}. Check that the path exists, is readable, and contains SKILL.md files.", e),
            }
        }
        "export-csv" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: medusa export-csv <path> <output.csv> [--no-cache]");
                return;
            }
            let path = &args[2];
            let output = &args[3];
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(path, true, use_cache) {
                Ok(result) => {
                    let snapshots: Vec<dream::SkillSnapshot> = result.skills.iter().map(dream::from_skill).collect();
                    dream::record_session(Path::new(path), &snapshots);
                    match export_csv(&result.skills, output) {
                        Ok(_) => eprintln!("CSV exported: {}", output),
                        Err(e) => eprintln!("Error writing CSV to '{}': {}. Check that the output path is writable.", output, e),
                    }
                }
                Err(e) => eprintln!("Scan failed: {}. Check that the path exists, is readable, and contains SKILL.md files.", e),
            }
        }
        "export-md" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: medusa export-md <path> <output.md> [--no-cache]");
                return;
            }
            let path = &args[2];
            let output = &args[3];
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(path, true, use_cache) {
                Ok(result) => {
                    let snapshots: Vec<dream::SkillSnapshot> = result.skills.iter().map(dream::from_skill).collect();
                    dream::record_session(Path::new(path), &snapshots);
                    match export_markdown(&result.skills, output) {
                        Ok(_) => eprintln!("Markdown exported: {}", output),
                        Err(e) => eprintln!("Error writing Markdown to '{}': {}. Check that the output path is writable.", output, e),
                    }
                }
                Err(e) => eprintln!("Scan failed: {}. Check that the path exists, is readable, and contains SKILL.md files.", e),
            }
        }
        "export-svg" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: medusa export-svg <path> <output.svg> [--no-cache]");
                return;
            }
            let path = &args[2];
            let output = &args[3];
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(path, true, use_cache) {
                Ok(result) => {
                    let snapshots: Vec<dream::SkillSnapshot> = result.skills.iter().map(dream::from_skill).collect();
                    dream::record_session(Path::new(path), &snapshots);
                    match export_svg(&result.skills, output) {
                        Ok(_) => eprintln!("SVG exported: {}", output),
                        Err(e) => eprintln!("Error writing SVG to '{}': {}. Check that the output path is writable.", output, e),
                    }
                }
                Err(e) => eprintln!("Scan failed: {}. Check that the path exists, is readable, and contains SKILL.md files.", e),
            }
        }
        "ab-test" => {
            if args.len() < 3 {
                eprintln!("Error: Missing path argument");
                eprintln!("Usage: medusa ab-test <path> [--iterations N]");
                return;
            }
            let path = &args[2];
            let mut iterations = 10;
            if let Some(pos) = args.iter().position(|a| a == "--iterations") {
                if let Some(val) = args.get(pos + 1) {
                    iterations = val.parse().unwrap_or(10);
                }
            }
            if let Err(e) = run_ab_test(path, iterations) {
                eprintln!("A/B test error: {}. Check that the path exists and contains SKILL.md files.", e);
            }
        }
        "audit" => {
            if args.len() < 3 {
                eprintln!("Error: Missing path argument");
                eprintln!("Usage: medusa audit <path> [--no-cache]");
                return;
            }
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(&args[2], true, use_cache) {
                Ok(result) => {
                    let path = Path::new(&args[2]);
                    let snapshots: Vec<dream::SkillSnapshot> = result.skills.iter().map(dream::from_skill).collect();
                    dream::record_session(path, &snapshots);
                    let dream_kb = Some(dream::load_knowledge_base_from_path(path));
                    print_audit_report(&result.skills, dream_kb.as_ref());

                    // Outcome assessments
                    let outcome_store = outcomes::load_outcomes(path);
                    if !outcome_store.rubrics.is_empty() {
                        println!("\n--- Outcome Assessments ---");
                        for skill in &result.skills {
                            if let Some(assessment) = outcomes::assess_skill(
                                &skill.id,
                                skill.metrics.content_length,
                                skill.metrics.code_blocks,
                                skill.metrics.step_count,
                                skill.metrics.tech_term_count,
                                &outcome_store,
                            ) {
                                outcomes::print_outcome_assessment(&assessment);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Audit failed: {}. Check that the path exists and contains SKILL.md files.", e),
            }
        }
        "dream" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let config = load_config(Path::new(path));
            let kb = dream::run_dream_with_config(Path::new(path), Some(&config.dreaming));
            dream::print_dream_report(&kb);
        }
        "dream-status" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let kb = dream::load_knowledge_base_from_path(Path::new(path));
            dream::print_dream_report(&kb);
        }
        "dream-reset" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let p = Path::new(path);
            let dream_path = dream::get_dream_path(p);
            let history_path = dream::get_history_path(p);
            let had_dream = dream_path.exists();
            let had_history = history_path.exists();
            if had_dream { let _ = fs::remove_file(&dream_path); }
            if had_history { let _ = fs::remove_file(&history_path); }
            if had_dream || had_history {
                eprintln!("Dream state and history reset.");
            } else {
                eprintln!("No dream state or history found at '{}'. Nothing to reset.", path);
            }
        }
        "dream-consolidate" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let config = load_config(Path::new(path));
            let mut kb = dream::load_knowledge_base(Path::new(path));
            let report = dream::consolidate_with_config(&mut kb, Some(&config.dreaming));
            dream::save_knowledge_base(Path::new(path), &kb);
            dream::print_consolidation_report(&report);
            eprintln!("Knowledge base consolidated and saved.");
        }
        "dream-params" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let p = Path::new(path);
            let config = load_config(p);
            println!("\n=== Dreaming Configuration ===");
            println!("  Config File: {}", p.join("medusa.toml").display());
            println!("  Frequency: Every {} scan(s)", config.dreaming.frequency_scans);
            println!("  Retention: {:.0}%", config.dreaming.retention_percent * 100.0);
            println!("  Auto-Apply: {}", config.dreaming.auto_apply);
            println!("  Max Insights: {}", config.dreaming.max_insights);
            println!("\n  Tip: Edit medusa.toml to change these values.");
        }
        "dream-diary" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let has_output = args.iter().position(|a| a == "--output");
            let diary = dream::generate_dream_diary(Path::new(path));
            if let Some(pos) = has_output {
                if let Some(output_path) = args.get(pos + 1) {
                    let md = dream::export_dream_diary_md(&diary);
                    match std::fs::write(output_path, md) {
                        Ok(_) => eprintln!("Dream diary exported to {}", output_path),
                        Err(e) => eprintln!("Error writing diary: {}", e),
                    }
                }
            } else {
                dream::print_dream_diary(&diary);
            }
        }
        "procedural-list" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let mem = procedural::load_procedural(Path::new(path));
            procedural::print_all_workflows(&mem);
        }
        "procedural-show" => {
            if args.len() < 4 {
                eprintln!("Usage: medusa procedural-show <path> <skill_id>");
                return;
            }
            let path = Path::new(&args[2]);
            let skill_id = &args[3];
            let mem = procedural::load_procedural(path);
            let workflows = procedural::get_workflows_for_skill(&mem, skill_id);
            if workflows.is_empty() {
                println!("No procedural workflows associated with '{}'", skill_id);
                println!("Run a scan to auto-detect workflows from skill step sequences.");
            } else {
                println!("\n=== Procedural Workflows for '{}' ===", skill_id);
                for w in workflows {
                    procedural::print_workflow(w);
                }
            }
        }
        "memory-export" => {
            if args.len() < 4 {
                eprintln!("Usage: medusa memory-export <path> <output.json>");
                return;
            }
            let path = Path::new(&args[2]);
            let output = &args[3];
            let dreaming = dream::load_knowledge_base_from_path(path);
            let procedural = procedural::load_procedural(path);
            let outcomes = outcomes::load_outcomes(path);
            let bundle = SharedMemoryBundle {
                source: format!("medusa@{}", std::env::current_exe().unwrap_or_default().display()),
                exported_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                dreaming,
                procedural,
                outcomes,
            };
            match serde_json::to_string_pretty(&bundle) {
                Ok(json) => match std::fs::write(output, &json) {
                    Ok(_) => eprintln!("Memory bundle exported to {} ({} KB)", output, json.len() / 1024),
                    Err(e) => eprintln!("Error writing bundle to '{}': {}. Check that the output path is writable.", output, e),
                },
                Err(e) => eprintln!("Error serializing bundle: {}. This may indicate corrupted memory state.", e),
            }
        }
        "memory-import" => {
            if args.len() < 4 {
                eprintln!("Usage: medusa memory-import <path> <input.json> [--source <name>]");
                return;
            }
            let path = Path::new(&args[2]);
            let input = &args[3];
            let source = args.iter().position(|a| a == "--source")
                .and_then(|p| args.get(p + 1)).cloned()
                .unwrap_or_else(|| "shared".to_string());

            match std::fs::read_to_string(input) {
                Ok(json) => match serde_json::from_str::<SharedMemoryBundle>(&json) {
                    Ok(bundle) => {
                        // Merge dreaming
                        let mut local_kb = dream::load_knowledge_base(path);
                        let before_dream = local_kb.insights.len();
                        for insight in &bundle.dreaming.insights {
                            let mut merged = insight.clone();
                            merged.metadata.insert("source".to_string(), source.clone());
                            if !local_kb.insights.iter().any(|i| i.id == merged.id) {
                                local_kb.insights.push(merged);
                            }
                        }
                        local_kb.total_patterns_found = local_kb.insights.len();
                        dream::save_knowledge_base(path, &local_kb);

                        // Merge procedural
                        let mut local_proc = procedural::load_procedural(path);
                        let before_proc = local_proc.workflows.len();
                        for w in &bundle.procedural.workflows {
                            if !local_proc.workflows.iter().any(|existing| existing.name == w.name) {
                                local_proc.workflows.push(w.clone());
                            }
                        }
                        procedural::save_procedural(path, &local_proc);

                        // Merge outcomes
                        let mut local_out = outcomes::load_outcomes(path);
                        let before_out = local_out.rubrics.len();
                        for (id, rubric) in &bundle.outcomes.rubrics {
                            if !local_out.rubrics.contains_key(id) {
                                local_out.rubrics.insert(id.clone(), rubric.clone());
                            }
                        }
                        outcomes::save_outcomes(path, &local_out);

                        eprintln!("Memory imported from '{}' (source: {})", input, source);
                        eprintln!("  Dream insights: {} → {} ({} new)", before_dream, local_kb.insights.len(), local_kb.insights.len() - before_dream);
                        eprintln!("  Workflows: {} → {} ({} new)", before_proc, local_proc.workflows.len(), local_proc.workflows.len() - before_proc);
                        eprintln!("  Rubrics: {} → {} ({} new)", before_out, local_out.rubrics.len(), local_out.rubrics.len() - before_out);
                    }
                    Err(e) => eprintln!("Error parsing bundle '{}': {}. The file may be corrupted or from an incompatible version.", input, e),
                },
                Err(e) => eprintln!("Error reading bundle file '{}': {}. Check that the file exists and is readable.", input, e),
            }
        }
        "outcome-add" => {
            if args.len() < 4 {
                eprintln!("Usage: medusa outcome-add <path> <skill_id>");
                return;
            }
            let path = Path::new(&args[2]);
            let skill_id = &args[3];
            let rubric = outcomes::get_default_rubric(skill_id);
            outcomes::add_rubric(path, rubric);
            eprintln!("Outcome rubric added for skill '{}'", skill_id);
        }
        "outcome-list" => {
            let path = if args.len() >= 3 { &args[2] } else { "." };
            let store = outcomes::load_outcomes(Path::new(path));
            outcomes::print_rubric_list(&store);
        }
        "outcome-remove" => {
            if args.len() < 4 {
                eprintln!("Usage: medusa outcome-remove <path> <skill_id>");
                return;
            }
            let path = Path::new(&args[2]);
            let skill_id = &args[3];
            if outcomes::remove_rubric(path, skill_id) {
                eprintln!("Rubric removed for '{}'", skill_id);
            } else {
                eprintln!("No rubric found for '{}'", skill_id);
            }
        }
        "learning-path" => {
            if args.len() < 4 {
                eprintln!("Usage: medusa learning-path <path> <skill_id>");
                return;
            }
            let path = Path::new(&args[2]);
            let skill_id = &args[3];
            let kb = dream::load_knowledge_base_from_path(path);
            let learning = dream::get_learning_path_for_skill(skill_id, &kb);
            dream::print_learning_path(skill_id, &learning);
            println!("\nTip: Run 'medusa audit <path>' for full metrics and outcome assessment.");
        }
        "orchestrate" => {
            if args.len() < 3 {
                eprintln!("Error: Missing path argument");
                eprintln!("Usage: medusa orchestrate <path> [--sequential] [--no-cache]");
                return;
            }
            let path = &args[2];
            let sequential = args.iter().any(|a| a == "--sequential");
            let use_cache = !args.iter().any(|a| a == "--no-cache");
            match scan_skills(path, !sequential, use_cache) {
                Ok(result) => {
                    let audits = agents::run_orchestrated_audit_all(&result.skills, &result.contents);
                    for audit in &audits {
                        agents::print_orchestrated_audit(audit);
                    }
                }
                Err(e) => eprintln!("Orchestrated audit failed: {}. Check that the path exists and contains SKILL.md files.", e),
            }
        }
        "update" => {
            eprintln!("Updating Medusa from GitHub...");
            // Get the directory where medusa.exe is located
            let exe_path = std::env::current_exe().unwrap_or_default();
            let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
            let repo_dir = if exe_dir.ends_with("release") || exe_dir.ends_with("debug") {
                exe_dir.parent().unwrap_or(Path::new(".")).parent().unwrap_or(Path::new("."))
            } else {
                exe_dir
            };
            
            match std::process::Command::new("git")
                .args(["-C", repo_dir.to_str().unwrap_or("."), "pull", "https://github.com/jtshow/medusa.git"])
                .status()
            {
                Ok(status) if status.success() => {
                    eprintln!("✅ Pull successful, rebuilding...");
                    match std::process::Command::new("cargo")
                        .args(["build", "--release"])
                        .current_dir(repo_dir)
                        .status()
                    {
                        Ok(status) if status.success() => eprintln!("✅ Medusa updated to latest version!"),
                        Ok(_) => eprintln!("❌ Build failed. Run 'cargo build --release' manually to see detailed errors."),
                        Err(e) => eprintln!("❌ Build error: {}. Ensure Rust (cargo) is installed and in your PATH.", e),
                    }
                }
                Ok(_) => eprintln!("❌ Git pull failed. Check your network connection or run 'git pull' manually."),
                Err(e) => eprintln!("❌ Git error: {}. Ensure git is installed and the repository exists.", e),
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_help();
        }
    }
}

// TODO: Implement learning paths, configurable scoring, enhanced suggestions, more export formats

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal MedusaConfig for testing.
    fn test_config() -> MedusaConfig {
        MedusaConfig {
            complexity_weight: 0.6,
            value_weight: 0.3,
            keyword_weight: 0.1,
            tier_thresholds: HashMap::new(),
            dreaming: DreamingConfig::default(),
        }
    }

    // ─── analyze_skill_complexity ────────────────────────────────────────────

    #[test]
    fn test_analyze_skill_complexity_empty_content() {
        let metrics = analyze_skill_complexity("");
        assert_eq!(metrics.content_length, 0);
        assert_eq!(metrics.code_blocks, 0);
        assert_eq!(metrics.step_count, 0);
        assert_eq!(metrics.tech_term_count, 0);
        assert_eq!(metrics.complexity_score, 0.0);
        assert_eq!(metrics.value_score, 50.0); // base value with no bonuses
    }

    #[test]
    fn test_analyze_skill_complexity_tech_terms() {
        let content = "This skill covers algorithm design, implementation patterns, and database architecture.";
        let metrics = analyze_skill_complexity(content);
        // algorithm, implementation, database, architecture = 4 tech terms
        assert_eq!(metrics.tech_term_count, 4);
        assert!(!metrics.complexity_score.is_nan());
        assert!(metrics.complexity_score > 0.0);
    }

    #[test]
    fn test_analyze_skill_complexity_code_blocks() {
        let content = "\
Some intro text here.

```python
print('hello')
```

More text.

```rust
fn main() {}
```

```javascript
console.log('hi');
```

End text.";
        let metrics = analyze_skill_complexity(content);
        assert_eq!(metrics.code_blocks, 3);
        assert!(metrics.complexity_score >= 15.0); // 3 * 5 = 15 from code blocks alone
    }

// ─── step_count test ── content uses (?m) regex, steps are "1." "2." etc."
    #[test]
    fn test_analyze_skill_complexity_step_count() {
        let content = "\
1. First step
2. Second step
3. Third step
4. Fourth step
5. Fifth step
6. Sixth step
7. Seventh step
8. Eighth step
9. Ninth step
10. Tenth step";
        let metrics = analyze_skill_complexity(content);
        assert_eq!(metrics.step_count, 10);
        assert!(metrics.complexity_score >= 20.0); // 10*2 = 20 from steps alone
    }

    #[test]
    fn test_analyze_skill_complexity_bonus_all_components() {
        // Has code blocks, >5 steps, >3 tech terms, and >3000 chars for all bonuses
        let content = "\
Introduction to this comprehensive skill that covers everything you need.

## Technical Overview

This skill provides expertise in algorithm design, implementation, database architecture, framework usage, optimization strategies, and security best practices including encryption and authentication.

## Implementation Steps

1. Install all prerequisites and set up the development environment.
2. Configure the database and connection strings.
3. Write the main implementation code.
4. Add unit tests for every function.
5. Run integration tests in CI.
6. Deploy to staging.
7. Verify with smoke tests.
8. Promote to production.
9. Monitor with observability tools.
10. Document everything.
11. Review and iterate.

```python
import sys
def main():
    pass
```

```rust
fn main() {}
```

```sql
CREATE TABLE users (id INT);
```

Conclusion and further reading.";
        let metrics = analyze_skill_complexity(content);
        assert!(metrics.complexity_score >= 10.0); // should have the +10 bonus
        assert!(metrics.complexity_score <= 100.0);
    }

    #[test]
    fn test_analyze_skill_complexity_capped_at_100() {
        // Massively over-specified content — score must cap at 100
        let mut content = String::new();
        for i in 0..200 {
            content.push_str(&format!("{}. Very long step description with lots of text to fill space\n", i + 1));
        }
        for _ in 0..30 {
            content.push_str("\n```python\nx = 1\n```\n");
        }
        // Add tons of tech terms
        for _ in 0..50 {
            content.push_str(" algorithm implementation architecture framework optimization");
        }
        let metrics = analyze_skill_complexity(&content);
        assert!(metrics.complexity_score <= 100.0);
    }

    // ─── calculate_experience ────────────────────────────────────────────────

    #[test]
    fn test_calculate_experience_base() {
        let metrics = SkillMetrics {
            content_length: 0,
            code_blocks: 0,
            step_count: 0,
            tech_term_count: 0,
            complexity_score: 0.0,
            value_score: 0.0,
        };
        let config = test_config();
        let exp = calculate_experience(&metrics, "", &config);
        assert_eq!(exp, 10.0); // base experience
    }

    #[test]
    fn test_calculate_experience_with_keywords() {
        let metrics = SkillMetrics {
            content_length: 0,
            code_blocks: 0,
            step_count: 0,
            tech_term_count: 0,
            complexity_score: 0.0,
            value_score: 0.0,
        };
        let config = test_config();
        // "rust" keyword gives 5.0 * 0.1 = 0.5
        let exp = calculate_experience(&metrics, "This skill uses rust programming", &config);
        assert!(exp > 10.0);
        assert!(exp < 11.0);
    }

    #[test]
    fn test_calculate_experience_caps_at_100() {
        let metrics = SkillMetrics {
            content_length: 100000,
            code_blocks: 100,
            step_count: 100,
            tech_term_count: 100,
            complexity_score: 100.0,
            value_score: 100.0,
        };
        let config = test_config();
        let exp = calculate_experience(&metrics, "advanced expert senior rust python kubernetes aws machine learning", &config);
        assert!(exp <= 100.0);
    }

    #[test]
    fn test_calculate_experience_configurable_weights() {
        let metrics = SkillMetrics {
            content_length: 5000,
            code_blocks: 10,
            step_count: 15,
            tech_term_count: 10,
            complexity_score: 80.0,
            value_score: 70.0,
        };
        let mut config = test_config();
        config.complexity_weight = 1.0;
        config.value_weight = 0.0;
        config.keyword_weight = 0.0;
        let exp = calculate_experience(&metrics, "test", &config);
        // Should be 10 + 80.0 * 1.0 = 90.0
        assert!((exp - 90.0).abs() < 0.01);
    }

// ─── get_level ───────────────────────────────────────────────────────────

    #[test]
    fn test_get_level_godlike() {
        assert_eq!(get_level(99.0), "Godlike");
        assert_eq!(get_level(95.0), "Godlike");
    }

    #[test]
    fn test_get_level_unique() {
        assert_eq!(get_level(94.9), "Unique");
        assert_eq!(get_level(90.0), "Unique");
    }

    #[test]
    fn test_get_level_legendary() {
        assert_eq!(get_level(89.9), "Legendary");
        assert_eq!(get_level(85.0), "Legendary");
    }

    #[test]
    fn test_get_level_mythic() {
        assert_eq!(get_level(84.9), "Mythic");
        assert_eq!(get_level(80.0), "Mythic");
    }

    #[test]
    fn test_get_level_epic() {
        assert_eq!(get_level(79.9), "Epic");
        assert_eq!(get_level(75.0), "Epic");
    }

    #[test]
    fn test_get_level_ultra_rare() {
        assert_eq!(get_level(74.9), "Ultra Rare");
        assert_eq!(get_level(65.0), "Ultra Rare");
    }

    #[test]
    fn test_get_level_rare() {
        assert_eq!(get_level(64.9), "Rare");
        assert_eq!(get_level(55.0), "Rare");
    }

#[test]
    fn test_get_level_uncommon() {
        // 45 >= 45 → "Uncommon"; 25 >= 25 but 25 < 45 → "Common"
        assert_eq!(get_level(54.9), "Uncommon");
        assert_eq!(get_level(45.0), "Uncommon");
        assert_eq!(get_level(25.0), "Common");
    }

    #[test]
    fn test_get_level_common() {
        assert_eq!(get_level(24.9), "Poor");
        assert_eq!(get_level(0.0), "Poor");
        assert_eq!(get_level(-1.0), "Poor");
    }

    #[test]
    fn test_get_level_poor() {
        // Below 25 should be Poor
        assert_eq!(get_level(-1.0), "Poor");
        assert_eq!(get_level(0.0), "Poor");
        assert_eq!(get_level(24.9), "Poor");
    }

    #[test]
    fn test_get_level_boundary_values() {
        // Test exact boundaries
        assert_eq!(get_level(95.0), "Godlike");
        assert_eq!(get_level(90.0), "Unique");
        assert_eq!(get_level(85.0), "Legendary");
        assert_eq!(get_level(80.0), "Mythic");
        assert_eq!(get_level(75.0), "Epic");
        assert_eq!(get_level(65.0), "Ultra Rare");
        assert_eq!(get_level(55.0), "Rare");
        assert_eq!(get_level(45.0), "Uncommon");
        assert_eq!(get_level(25.0), "Common");
        assert_eq!(get_level(24.99), "Poor");
        assert_eq!(get_level(-1.0), "Poor");
    }

// ─── calculate_confidence ────────────────────────────────────────────────

    #[test]
    fn test_calculate_confidence_above_100_chars() {
        let metrics = SkillMetrics {
            content_length: 150,
            code_blocks: 0,
            step_count: 0,
            tech_term_count: 0,
            complexity_score: 0.0,
            value_score: 0.0,
        };
        let conf = calculate_confidence("desc", "label", &metrics);
        // base 0.3 + 0.2 for >100 chars = 0.5 exactly
        assert!((conf - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_confidence_maximum() {
        let metrics = SkillMetrics {
            content_length: 10000,
            code_blocks: 50,
            step_count: 100,
            tech_term_count: 50,
            complexity_score: 100.0,
            value_score: 100.0,
        };
        let conf = calculate_confidence("a description", "a label", &metrics);
        assert!(conf <= 1.0);
    }

#[test]
    fn test_calculate_confidence_minimum() {
        let metrics = SkillMetrics {
            content_length: 0,
            code_blocks: 0,
            step_count: 0,
            tech_term_count: 0,
            complexity_score: 0.0,
            value_score: 0.0,
        };
        let conf = calculate_confidence("desc", "label", &metrics);
        // base 0.3 with no bonuses, clamped to min 0.0
        assert!(conf >= 0.0);
        assert!(conf <= 1.0);
    }

    // ─── string_similarity ───────────────────────────────────────────────────

    #[test]
    fn test_string_similarity_identical() {
        assert_eq!(string_similarity("hello world", "hello world"), 1.0);
    }

    #[test]
    fn test_string_similarity_empty() {
        assert_eq!(string_similarity("", "hello"), 0.0);
        assert_eq!(string_similarity("hello", ""), 0.0);
        assert_eq!(string_similarity("", ""), 0.0);
    }

    #[test]
    fn test_string_similarity_partial_overlap() {
        let sim = string_similarity("rust programming", "rust language");
        // shared: "rust" → common=1, total=4, score = 2*1/4 = 0.5
        assert!((sim - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_string_similarity_no_overlap() {
        let sim = string_similarity("aaaa bbbb", "cccc dddd");
        assert_eq!(sim, 0.0);
    }

    // ─── detect_fusion ────────────────────────────────────────────────────────

    #[test]
    fn test_detect_fusion_high_similarity() {
        // Use near-identical labels & descriptions to ensure similarity > 0.5
        let skills = vec![
            Skill {
                id: "skill-a".to_string(),
                label: "rust programming language".to_string(),
                description: "Learn rust programming language from scratch".to_string(),
                experience: 50.0,
                level: "Epic".to_string(),
                confidence: 0.8,
                metrics: SkillMetrics {
                    content_length: 1000,
                    code_blocks: 5,
                    step_count: 10,
                    tech_term_count: 10,
                    complexity_score: 50.0,
                    value_score: 60.0,
                },
                context: SkillContext::default(),
            },
            Skill {
                id: "skill-b".to_string(),
                label: "rust language".to_string(),
                description: "The rust programming language guide".to_string(),
                experience: 40.0,
                level: "Epic".to_string(),
                confidence: 0.7,
                metrics: SkillMetrics {
                    content_length: 800,
                    code_blocks: 4,
                    step_count: 8,
                    tech_term_count: 8,
                    complexity_score: 40.0,
                    value_score: 50.0,
                },
                context: SkillContext::default(),
            },
        ];
        let matches = detect_fusion(&skills);
        assert!(!matches.is_empty());
        assert!(matches[0].similarity > 0.5);
    }

    #[test]
    fn test_detect_fusion_no_similarity() {
        let skills = vec![
            Skill {
                id: "skill-a".to_string(),
                label: "zzzzz aaaaa".to_string(),
                description: "completely unrelated skill one".to_string(),
                experience: 50.0,
                level: "Epic".to_string(),
                confidence: 0.8,
                metrics: SkillMetrics {
                    content_length: 1000,
                    code_blocks: 5,
                    step_count: 10,
                    tech_term_count: 10,
                    complexity_score: 50.0,
                    value_score: 60.0,
                },
                context: SkillContext::default(),
            },
            Skill {
                id: "skill-b".to_string(),
                label: "qqqqq bbbbb".to_string(),
                description: "completely unrelated skill two".to_string(),
                experience: 40.0,
                level: "Epic".to_string(),
                confidence: 0.7,
                metrics: SkillMetrics {
                    content_length: 800,
                    code_blocks: 4,
                    step_count: 8,
                    tech_term_count: 8,
                    complexity_score: 40.0,
                    value_score: 50.0,
                },
                context: SkillContext::default(),
            },
        ];
        let matches = detect_fusion(&skills);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_detect_fusion_truncated_to_20() {
        // Create 25 skills that are all similar to each other
        let base_labels: Vec<String> = (0..25).map(|i| format!("test skill number {}", i)).collect();
        let skills: Vec<Skill> = base_labels.iter().enumerate().map(|(i, label)| Skill {
            id: format!("skill-{}", i),
            label: label.clone(),
            description: "a shared description for fusion testing".to_string(),
            experience: 50.0,
            level: "Epic".to_string(),
            confidence: 0.8,
            metrics: SkillMetrics {
                content_length: 1000,
                code_blocks: 5,
                step_count: 10,
                tech_term_count: 10,
                complexity_score: 50.0,
                value_score: 60.0,
            },
            context: SkillContext::default(),
        }).collect();
        let matches = detect_fusion(&skills);
        assert!(matches.len() <= 20);
    }

    // ─── build_learning_paths ────────────────────────────────────────────────

    #[test]
    fn test_build_learning_paths_single_category() {
        let skills = vec![
            Skill {
                id: "rust-basics".to_string(),
                label: "Rust Basics".to_string(),
                description: "".to_string(),
                experience: 30.0,
                level: "Common".to_string(),
                confidence: 0.7,
                metrics: SkillMetrics {
                    content_length: 500,
                    code_blocks: 2,
                    step_count: 3,
                    tech_term_count: 5,
                    complexity_score: 20.0,
                    value_score: 30.0,
                },
                context: SkillContext::default(),
            },
            Skill {
                id: "rust-advanced".to_string(),
                label: "Rust Advanced".to_string(),
                description: "".to_string(),
                experience: 75.0,
                level: "Epic".to_string(),
                confidence: 0.9,
                metrics: SkillMetrics {
                    content_length: 2000,
                    code_blocks: 10,
                    step_count: 15,
                    tech_term_count: 20,
                    complexity_score: 80.0,
                    value_score: 90.0,
                },
                context: SkillContext::default(),
            },
        ];
        let paths = build_learning_paths(&skills);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].skills, vec!["rust-basics", "rust-advanced"]);
    }

    #[test]
    fn test_build_learning_paths_single_skill_no_path() {
        let skills = vec![Skill {
            id: "lone-skill".to_string(),
            label: "Lone Skill".to_string(),
            description: "".to_string(),
            experience: 50.0,
            level: "Epic".to_string(),
            confidence: 0.8,
            metrics: SkillMetrics {
                content_length: 500,
                code_blocks: 2,
                step_count: 3,
                tech_term_count: 5,
                complexity_score: 20.0,
                value_score: 30.0,
            },
            context: SkillContext::default(),
        }];
        let paths = build_learning_paths(&skills);
        assert!(paths.is_empty()); // Single skill → no path built
    }

    #[test]
    fn test_build_learning_paths_sorted_by_experience() {
        let skills = vec![
            Skill {
                id: "rust-a".to_string(),
                label: "Rust A".to_string(),
                description: "".to_string(),
                experience: 80.0,
                level: "Mythic".to_string(),
                confidence: 0.9,
                metrics: SkillMetrics {
                    content_length: 1000,
                    code_blocks: 5,
                    step_count: 8,
                    tech_term_count: 10,
                    complexity_score: 60.0,
                    value_score: 70.0,
                },
                context: SkillContext::default(),
            },
            Skill {
                id: "rust-b".to_string(),
                label: "Rust B".to_string(),
                description: "".to_string(),
                experience: 20.0,
                level: "Common".to_string(),
                confidence: 0.6,
                metrics: SkillMetrics {
                    content_length: 300,
                    code_blocks: 1,
                    step_count: 2,
                    tech_term_count: 3,
                    complexity_score: 10.0,
                    value_score: 20.0,
                },
                context: SkillContext::default(),
            },
            Skill {
                id: "rust-c".to_string(),
                label: "Rust C".to_string(),
                description: "".to_string(),
                experience: 50.0,
                level: "Epic".to_string(),
                confidence: 0.8,
                metrics: SkillMetrics {
                    content_length: 600,
                    code_blocks: 3,
                    step_count: 5,
                    tech_term_count: 7,
                    complexity_score: 35.0,
                    value_score: 45.0,
                },
                context: SkillContext::default(),
            },
        ];
        let paths = build_learning_paths(&skills);
        assert_eq!(paths.len(), 1);
        // Should be sorted by experience: rust-b (20) → rust-c (50) → rust-a (80)
        assert_eq!(paths[0].skills, vec!["rust-b", "rust-c", "rust-a"]);
    }

    // ─── parse_skill_md ──────────────────────────────────────────────────────

    #[test]
    fn test_parse_skill_md_valid() {
        let content = "\
---
id: my-skill
name: My Skill
description: A test skill
---

## Steps

1. First step
2. Second step
3. Third step";
        let config = test_config();
        let skill = parse_skill_md(content, "test.md".as_ref(), &config);
        assert!(skill.is_some());
        let skill = skill.unwrap();
        assert_eq!(skill.id, "my-skill");
        assert_eq!(skill.label, "My Skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.metrics.step_count, 3);
    }

    #[test]
    fn test_parse_skill_md_no_frontmatter() {
        let content = "Just some text without front matter.";
        let config = test_config();
        let skill = parse_skill_md(content, "test.md".as_ref(), &config);
        assert!(skill.is_none());
    }

    #[test]
    fn test_parse_skill_md_missing_description() {
        let content = "\
---
id: no-desc
name: No Description
---

Some content here.";
        let config = test_config();
        let skill = parse_skill_md(content, "test.md".as_ref(), &config);
        assert!(skill.is_some());
        let skill = skill.unwrap();
        assert_eq!(skill.description, "");
    }

    #[test]
    fn test_parse_skill_md_fallback_id_from_filename() {
        let content = "\
---
description: A skill without explicit id
---

Content.";
        let config = test_config();
        let skill = parse_skill_md(content, "fallback-file.md".as_ref(), &config);
        assert!(skill.is_some());
        let skill = skill.unwrap();
        assert_eq!(skill.id, "fallback-file");
    }

    // ─── export_csv ──────────────────────────────────────────────────────────

    #[test]
    fn test_export_csv() {
        let skills = vec![Skill {
            id: "csv-test".to_string(),
            label: "CSV Test".to_string(),
            description: "A test for CSV export".to_string(),
            experience: 55.5,
            level: "Epic".to_string(),
            confidence: 0.75,
            metrics: SkillMetrics {
                content_length: 10000,
                code_blocks: 8,
                step_count: 15,
                tech_term_count: 12,
                complexity_score: 75.0,
                value_score: 85.0,
            },
            context: SkillContext {
                dependencies: vec![],
                fusion_opportunities: vec![],
                improvement_history: vec![],
                gaps: vec![],
            },
        }];
        let path = std::env::temp_dir().join("medusa_test_export.csv");
        let result = export_csv(&skills, path.to_str().unwrap());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("csv-test"));
        assert!(content.contains("55.5"));
        assert!(content.contains("Epic"));
        std::fs::remove_file(&path).ok();
    }

    // ─── export_markdown ─────────────────────────────────────────────────────

    #[test]
    fn test_export_markdown() {
        let skills = vec![Skill {
            id: "md-test".to_string(),
            label: "Markdown Test".to_string(),
            description: "A test for Markdown export".to_string(),
            experience: 60.0,
            level: "Epic".to_string(),
            confidence: 0.85,
            metrics: SkillMetrics {
                content_length: 5000,
                code_blocks: 10,
                step_count: 20,
                tech_term_count: 15,
                complexity_score: 85.0,
                value_score: 90.0,
            },
            context: SkillContext {
                dependencies: vec![],
                fusion_opportunities: vec![],
                improvement_history: vec![],
                gaps: vec!["Add more examples".to_string()],
            },
        }];
        let path = std::env::temp_dir().join("medusa_test_export.md");
        let result = export_markdown(&skills, path.to_str().unwrap());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Medusa Skill Report"));
        assert!(content.contains("Markdown Test"));
        assert!(content.contains("Add more examples"));
        std::fs::remove_file(&path).ok();
    }

    // ─── load_config ─────────────────────────────────────────────────────────

    #[test]
    fn test_load_config_default() {
        let temp_dir = std::env::temp_dir().join("medusa_config_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config = load_config(&temp_dir);
        assert_eq!(config.complexity_weight, 0.6);
        assert_eq!(config.value_weight, 0.3);
        assert_eq!(config.keyword_weight, 0.1);
        // Clean up
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_load_config_from_file() {
        let temp_dir = std::env::temp_dir().join("medusa_config_test_file");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let toml_content = r#"
complexity_weight = 0.8
value_weight = 0.15
keyword_weight = 0.05

[dreaming]
frequency_scans = 3
retention_percent = 0.9
auto_apply = false
max_insights = 100
"#;
        std::fs::write(temp_dir.join("medusa.toml"), toml_content).unwrap();
        let config = load_config(&temp_dir);
        assert!((config.complexity_weight - 0.8).abs() < f64::EPSILON);
        assert!((config.value_weight - 0.15).abs() < f64::EPSILON);
        assert!((config.keyword_weight - 0.05).abs() < f64::EPSILON);
        assert_eq!(config.dreaming.frequency_scans, 3);
        assert!((config.dreaming.retention_percent - 0.9).abs() < f64::EPSILON);
        assert!(!config.dreaming.auto_apply);
        assert_eq!(config.dreaming.max_insights, 100);
        // Clean up
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    // ─── scan edge cases ─────────────────────────────────────────────────────

    #[test]
    fn test_scan_nonexistent_path() {
        let result = scan_skills("/nonexistent/path/12345", false, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.total, 0);
        assert!(result.skills.is_empty());
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = std::env::temp_dir().join("medusa_empty_scan_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let result = scan_skills(temp_dir.to_str().unwrap(), false, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.total, 0);
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    // ─── ScanResult serialization ────────────────────────────────────────────

    #[test]
    fn test_scan_result_serialization() {
        let result = ScanResult {
            skills: vec![],
            total: 0,
            scan_time_ms: 42,
            fusion_matches: vec![],
            rust_used: true,
            version: "0.12.1".to_string(),
            scan_type: "parallel".to_string(),
            learning_paths: vec![],
            dream_summary: None,
            skill_outcomes: None,
            contents: HashMap::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total\":0"));
        assert!(json.contains("\"scan_time_ms\":42"));
    }

    // ─── Skill struct Default ────────────────────────────────────────────────

    #[test]
    fn test_skill_default_values() {
        let skill = Skill {
            id: "test".to_string(),
            label: "Test".to_string(),
            description: "Test skill".to_string(),
            experience: 0.0,
            level: get_level(0.0),
            confidence: 0.0,
            metrics: SkillMetrics {
                content_length: 0,
                code_blocks: 0,
                step_count: 0,
                tech_term_count: 0,
                complexity_score: 0.0,
                value_score: 0.0,
            },
            context: SkillContext::default(),
        };
        assert_eq!(skill.level, "Poor");
        assert!(skill.context.dependencies.is_empty());
        assert!(skill.context.gaps.is_empty());
    }
}
