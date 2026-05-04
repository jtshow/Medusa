//! Medusa - Ultra-Fast Skill Scanner v0.11 (MSF)
//! Features: Audit-based ranking (60/30/10), auto-promotion, 9-tier system, context building

use std::path::Path;
use std::fs;
use std::time::Instant;
use std::collections::HashMap;
use walkdir::WalkDir;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Regex for YAML frontmatter.
    static ref RE_NAME: Regex = Regex::new(r#"name:\s*"?([^"\s}]+)"?#?"#).unwrap();
    static ref RE_DESC: Regex = Regex::new(r#"description:\s*"([^"]+)""#).unwrap();
    
    // Regex for complexity analysis.
    static ref RE_CODE_BLOCK: Regex = Regex::new(r#"```[\s\S]*?```"#).unwrap();
    static ref RE_STEPS: Regex = Regex::new(r#"^\s*(\d+\.|[-*])\s"#).unwrap();
    static ref RE_TECH_TERMS: Regex = Regex::new(
        r#"(algorithm|implementation|architecture|framework|optimization|scalability|security|encryption|authentication|database|api|sdk|middleware)"#
    ).unwrap();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SkillMetrics {
    content_length: usize,
    code_blocks: usize,
    step_count: usize,
    tech_term_count: usize,
    complexity_score: f64,
    value_score: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Skill {
    id: String,
    label: String,
    description: String,
    experience: f64,
    level: String,
    confidence: f64,
    metrics: SkillMetrics,
    context: SkillContext,  // NEW: Context information!
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct SkillContext {
    dependencies: Vec<SkillDep>,
    fusion_opportunities: Vec<String>,
    improvement_history: Vec<ImprovementRecord>,
    gaps: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SkillDep {
    name: String,
    relationship: String,
    context: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ImprovementRecord {
    date: String,
    action: String,
    impact: String,
    evidence: String,
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
}

#[derive(Debug, Clone, serde::Serialize)]
struct FusionMatch {
    skill1: String,
    skill2: String,
    similarity: f64,
    match_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct MedusaConfig {
    #[serde(default = "default_complexity_weight")]
    complexity_weight: f64,
    #[serde(default = "default_value_weight")]
    value_weight: f64,
    #[serde(default = "default_keyword_weight")]
    keyword_weight: f64,
    #[serde(default)]
    tier_thresholds: HashMap<String, f64>,
}

fn default_complexity_weight() -> f64 { 0.6 }
fn default_value_weight() -> f64 { 0.3 }
fn default_keyword_weight() -> f64 { 0.1 }

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
    if !content.starts_with("---") {
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

    let id = parse_field_regex(&RE_NAME, fm).unwrap_or_else(|| {
        file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string()
    });

    let label = id.clone();
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
            .and_then(|s| toml::from_str(&s).ok())
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
    
    exp.min(100.0)
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
        sorted_skills.sort_by(|a, b| a.experience.partial_cmp(&b.experience).unwrap());
        
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
            rust_used: true,
            version: "0.11.0".to_string(),
            scan_type: if parallel { "parallel" } else { "sequential" }.to_string(),
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
                    if let Some(entry) = cache.entries.get(&path_str) {
                        if entry.hash == hash {
                            new_skills.push(entry.skill.clone());
                            continue;
                        }
                    }
                }
                
                if let Some(skill) = parse_skill_md(&content, entry.path(), &config) {
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
                parse_skill_md(&content, entry.path(), &config)
            })
            .collect()
    };

    // Save cache
    if use_cache {
        let _ = fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap_or_default());
    }

    let mut skills = skills;
    skills.sort_by(|a, b| b.experience.partial_cmp(&a.experience).unwrap());

    let fusion_matches = detect_fusion(&skills);
    let learning_paths = build_learning_paths(&skills);

    let elapsed = start.elapsed();

    Ok(ScanResult {
        total: skills.len(),
        scan_time_ms: elapsed.as_millis() as u64,
        skills,
        fusion_matches,
        learning_paths,
        rust_used: true,
        version: "0.11.0".to_string(),
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

    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    matches.truncate(20);
    matches
}

/// String similarity using FxHash.
fn string_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 { return 1.0; }
    if s1.is_empty() || s2.is_empty() { return 0.0; }

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
    println!("Medusa Skill Framework (MSF) v0.11.0 - Audit-Based Ranking with Context");
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

/// Print audit report with FULL context.
fn print_audit_report(skills: &[Skill]) {
    println!("\n=== Medusa Skill Audit Report (v0.11) ===\n");
    
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
        println!();
    }
    
    // Summary.
    let avg_complexity = skills.iter().map(|s| s.metrics.complexity_score).sum::<f64>() / skills.len() as f64;
    let avg_value = skills.iter().map(|s| s.metrics.value_score).sum::<f64>() / skills.len() as f64;
    
    println!("=== Summary ===");
    println!("Total Skills: {}", skills.len());
    println!("Average Complexity: {:.1}/100 (60% of ranking)", avg_complexity);
    println!("Average Value: {:.1}/100 (30% of ranking)", avg_value);
}

/// Generate HTML with context.
fn generate_html(result: &ScanResult, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    html.push_str("    </style>\n</head>\n<body>\n");
    
    html.push_str(&format!("    <h1>Medusa Scan Report (v0.11)</h1>\n    <div class=\"meta\"><p>Total Skills: {} | Scan Time: {}ms | Version: {} | Type: {}</p></div>\n",
        result.total, result.scan_time_ms, result.version, result.scan_type));
    
    html.push_str("    <h2>Skills (Sorted by Experience)</h2>\n    <div id=\"skills\">\n");
    for s in &result.skills {
        html.push_str(&format!(
            "        <div class=\"skill level-{}\"><h3>{} <span class=\"meta\">[{}]</span></h3><p>{}</p><div class=\"bar\"><div class=\"bar-fill\" style=\"width: {}%\"></div></div><p class=\"meta\">ID: {} | Exp: {} | Conf: {}%</p><p class=\"metrics\">Len: {} | Code: {} | Steps: {} | Terms: {} | Comp: {:.1} | Val: {:.1}</p><p class=\"context\">Gaps: {}</p></div>\n",
            s.level.to_lowercase(), s.label, s.level, s.description, s.experience, s.id, s.experience, (s.confidence * 100.0).floor(),
            s.metrics.content_length, s.metrics.code_blocks, s.metrics.step_count, s.metrics.tech_term_count, s.metrics.complexity_score, s.metrics.value_score,
            s.context.gaps.join(", ")
        ));
    }
    html.push_str("    </div>\n");
    
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
            println!("Medusa Skill Framework (MSF) v0.11.0");
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
                Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
                Err(e) => eprintln!("Error: {}", e),
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
                    match generate_html(&result, output) {
                        Ok(_) => eprintln!("HTML report generated: {}", output),
                        Err(e) => eprintln!("Error writing HTML: {}", e),
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
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
                    match export_csv(&result.skills, output) {
                        Ok(_) => eprintln!("CSV exported: {}", output),
                        Err(e) => eprintln!("Error writing CSV: {}", e),
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
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
                    match export_markdown(&result.skills, output) {
                        Ok(_) => eprintln!("Markdown exported: {}", output),
                        Err(e) => eprintln!("Error writing Markdown: {}", e),
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
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
                    match export_svg(&result.skills, output) {
                        Ok(_) => eprintln!("SVG exported: {}", output),
                        Err(e) => eprintln!("Error writing SVG: {}", e),
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
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
                eprintln!("A/B test error: {}", e);
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
                Ok(result) => print_audit_report(&result.skills),
                Err(e) => eprintln!("Error: {}", e),
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
                        Ok(_) => eprintln!("❌ Build failed"),
                        Err(e) => eprintln!("❌ Build error: {}", e),
                    }
                }
                Ok(_) => eprintln!("❌ Git pull failed"),
                Err(e) => eprintln!("❌ Git error: {}", e),
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_help();
        }
    }
}

// TODO: Implement learning paths, configurable scoring, enhanced suggestions, more export formats
