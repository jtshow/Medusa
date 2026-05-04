//! Medusa - Ultra-Fast Skill Scanner v0.11 (MSF)
//! Features: Audit-based ranking (60/30/10), auto-promotion, 9-tier system, context building

use std::path::Path;
use std::fs;
use std::time::Instant;
use rayon::prelude::*;
use walkdir::WalkDir;
use serde_json;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_NAME: Regex = Regex::new(r#"name:\s*"?([^"\s}]+)"?#?"#).unwrap();
    static ref RE_DESC: Regex = Regex::new(r#"description:\s*"([^"]+)""#).unwrap();
    static ref RE_CODE_BLOCK: Regex = Regex::new(r#"```[\s\S]*?```"#).unwrap();
    static ref RE_STEPS: Regex = Regex::new(r#"^\s*(\d+\.|[-*])\s"#).unwrap();
    static ref RE_TECH_TERMS: Regex = Regex::new(
        r#"(algorithm|implementation|architecture|framework|optimization|scalability|security|encryption|authentication|database|api|sdk|middleware)"#
    ).unwrap();
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

#[derive(Debug, Clone, serde::Serialize)]
struct FusionMatch {
    skill1: String,
    skill2: String,
    similarity: f64,
    match_type: String,
}

#[derive(Debug, serde::Serialize)]
struct ScanResult {
    skills: Vec<Skill>,
    total: usize,
    scan_time_ms: u64,
    fusion_matches: Vec<FusionMatch>,
    rust_used: bool,
    version: String,
    scan_type: String,
}

fn print_help() {
    println!("Medusa Skill Framework (MSF) v0.11.0 - Audit-Based Ranking with Context");
    println!("Usage: medusa <command> [options]");
    println!("\nCommands:");
    println!("  scan <path>              Scan skills with audit-based ranking");
    println!("    --sequential           Use sequential scanning (no Rayon)");
    println!("\n  html <path> <output>   Generate HTML visualization");
    println!("    --sequential           Use sequential scanning");
    println!("\n  ab-test <path>          Run A/B test (parallel vs sequential)");
    println!("    --iterations N         Number of test iterations (default: 10)");
    println!("\n  update                  Update Medusa from GitHub (git pull + rebuild)");
    println!("\n  audit <path>            Show detailed skill audit report");
    println!("\nOptions:");
    println!("  --help, -h              Show this help message");
    println!("  --version, -v           Show version");
    println!("\nExamples:");
    println!("  medusa scan /path/to/skills");
    println!("  medusa audit /path/to/skills  # Detailed complexity analysis");
    println!("  medusa html /path/to/skills report.html");
}

fn extract_frontmatter_str(content: &str) -> Option<&str> {
    if !content.starts_with("---") {
        return None;
    }
    let start = 4;
    content[start..].find("\n---").map(|pos| &content[start..start + pos])
}

fn parse_field_regex(re: &Regex, fm: &str) -> Option<String> {
    re.captures(fm).map(|cap| cap[1].to_string())
}

fn analyze_skill_complexity(content: &str) -> SkillMetrics {
    let content_length = content.len();
    
    // Count code blocks
    let code_blocks = RE_CODE_BLOCK.find_iter(content).count();
    
    // Count step-by-step instructions
    let step_count = RE_STEPS.find_iter(content).count();
    
    // Count technical terms
    let tech_term_count = RE_TECH_TERMS.find_iter(&content.to_lowercase()).count();
    
    // Calculate complexity score (0-100)
    let mut complexity = 0.0_f64;
    
    // Length factor (max 30 points)
    complexity += (content_length as f64 / 100.0).min(30.0);
    
    // Code blocks factor (max 25 points)
    complexity += (code_blocks as f64 * 5.0).min(25.0);
    
    // Steps factor (max 20 points)
    complexity += (step_count as f64 * 2.0).min(20.0);
    
    // Technical terms factor (max 25 points)
    complexity += (tech_term_count as f64 * 2.5).min(25.0);
    
    // Bonus for having all components
    if code_blocks > 0 && step_count > 5 && tech_term_count > 3 {
        complexity += 10.0;
    }
    
    complexity = complexity.min(100.0);
    
    // Calculate value score based on content quality
    let mut value: f64 = 50.0; // Base value
    
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

fn parse_skill_md(content: &str, file_path: &Path) -> Option<Skill> {
    let fm = extract_frontmatter_str(content)?;

    let id = parse_field_regex(&RE_NAME, fm).unwrap_or_else(|| {
        file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string()
    });

    let label = id.clone();
    let description = parse_field_regex(&RE_DESC, fm).unwrap_or_default();
    
    // Audit-based experience calculation
    let metrics = analyze_skill_complexity(content);
    let experience = calculate_experience(&metrics, &description);
    
    let level = get_level(experience);
    let confidence = calculate_confidence(&description, &label, &metrics);

    Some(Skill {
        id,
        label,
        description,
        experience,
        level,
        confidence,
        metrics,
    })
}

fn calculate_experience(metrics: &SkillMetrics, description: &str) -> f64 {
    let mut exp = 10.0; // Base experience
    
    // Complexity-based scoring (primary factor)
    exp += metrics.complexity_score * 0.6;
    
    // Value-based scoring
    exp += metrics.value_score * 0.3;
    
    // Description keyword bonuses
    let desc_lower = description.to_lowercase();
    let keyword_bonuses = [
        ("advanced", 8.0), ("expert", 12.0), ("senior", 10.0),
        ("react", 5.0), ("vue", 5.0), ("angular", 5.0),
        ("security", 8.0), ("owasp", 10.0), ("penetration", 10.0),
        ("rust", 8.0), ("python", 5.0), ("javascript", 4.0),
        ("kubernetes", 10.0), ("docker", 6.0), ("aws", 8.0),
        ("machine learning", 12.0), ("ai", 8.0), ("llm", 10.0),
    ];
    
    for (kw, score) in keyword_bonuses {
        if desc_lower.contains(kw) {
            exp += score;
        }
    }
    
    exp.min(100.0)
}

fn get_level(exp: f64) -> String {
    match exp {
        e if e >= 95.0 => "Godlike",      // 95+ (was 85+)
        e if e >= 90.0 => "Unique",        // 90+
        e if e >= 85.0 => "Legendary",     // 85+
        e if e >= 80.0 => "Mythic",        // 80+
        e if e >= 75.0 => "Epic",          // 75+
        e if e >= 65.0 => "Ultra Rare",    // 65+ (was 65+ Expert)
        e if e >= 55.0 => "Rare",          // 55+ (was 60+ Expert)
        e if e >= 45.0 => "Uncommon",      // 45+ (was 45+ Advanced)
        e if e >= 25.0 => "Common",        // 25+ (was 25+ Intermediate)
        _ => "Poor",                    // <25 (was Beginner)
    }.to_string()
}

fn calculate_confidence(description: &str, _label: &str, metrics: &SkillMetrics) -> f64 {
    let mut conf: f64 = 0.3;
    
    // Length factor
    if description.len() > 100 { conf += 0.2; }
    if description.len() > 300 { conf += 0.15; }
    
    // Content quality factors
    if metrics.code_blocks > 0 { conf += 0.15; }
    if metrics.step_count > 5 { conf += 0.1; }
    if metrics.tech_term_count > 3 { conf += 0.1; }
    
    conf.min(1.0)
}

fn scan_skills(path: &str, parallel: bool) -> Result<ScanResult, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let path = Path::new(path);

    if !path.is_dir() {
        return Ok(ScanResult {
            skills: vec![],
            total: 0,
            scan_time_ms: 0,
            fusion_matches: vec![],
            rust_used: true,
             version: "0.11.0".to_string(),
            scan_type: if parallel { "parallel" } else { "sequential" }.to_string(),
        });
    }

    let entries: Vec<_> = WalkDir::new(path)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().file_name().map_or(false, |n| n == "SKILL.md"))
        .collect();

    let skills: Vec<_> = if parallel {
        entries
            .par_iter()
            .filter_map(|entry| {
                let content = fs::read_to_string(entry.path()).ok()?;
                parse_skill_md(&content, entry.path())
            })
            .collect()
    } else {
        entries
            .iter()
            .filter_map(|entry| {
                let content = fs::read_to_string(entry.path()).ok()?;
                parse_skill_md(&content, entry.path())
            })
            .collect()
    };

    let mut skills = skills;
    skills.sort_by(|a, b| b.experience.partial_cmp(&a.experience).unwrap());

    let fusion_matches = detect_fusion(&skills);

    let elapsed = start.elapsed();

    Ok(ScanResult {
        total: skills.len(),
        scan_time_ms: elapsed.as_millis() as u64,
        skills,
        fusion_matches,
        rust_used: true,
        version: "0.6.0".to_string(),
        scan_type: if parallel { "parallel" } else { "sequential" }.to_string(),
    })
}

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

fn generate_html(result: &ScanResult, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n    <title>Medusa Scan Report</title>\n    <style>\n");
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
    html.push_str("    </style>\n</head>\n<body>\n");
    
    html.push_str(&format!("    <h1>Medusa Scan Report</h1>\n    <div class=\"meta\"><p>Total Skills: {} | Scan Time: {}ms | Version: {} | Type: {}</p></div>\n",
        result.total, result.scan_time_ms, result.version, result.scan_type));
    
    html.push_str("    <h2>Skills (Sorted by Experience)</h2>\n    <div id=\"skills\">\n");
    for s in &result.skills {
        html.push_str(&format!(
            "        <div class=\"skill level-{}\"><h3>{} <span class=\"meta\">[{}]</span></h3><p>{}</p><div class=\"bar\"><div class=\"bar-fill\" style=\"width: {}%\"></div></div><p class=\"meta\">ID: {} | Exp: {} | Conf: {}%</p><p class=\"metrics\">Length: {} | Code: {} | Steps: {} | Tech Terms: {} | Complexity: {:.1} | Value: {:.1}</p></div>\n",
            s.level.to_lowercase(), s.label, s.level, s.description, s.experience, s.id, s.experience, (s.confidence * 100.0).floor(),
            s.metrics.content_length, s.metrics.code_blocks, s.metrics.step_count, s.metrics.tech_term_count, s.metrics.complexity_score, s.metrics.value_score
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

fn run_ab_test(path: &str, iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Running A/B Test: Parallel vs Sequential Scan");
    eprintln!("Path: {}", path);
    eprintln!("Iterations: {}", iterations);
    eprintln!("\nHypothesis: Parallel scanning is faster than sequential");
    eprintln!("Primary metric: scan_time_ms\n");

    let mut parallel_times = Vec::new();
    let mut sequential_times = Vec::new();

    for i in 1..=iterations {
        let parallel_result = scan_skills(path, true)?;
        parallel_times.push(parallel_result.scan_time_ms);

        let sequential_result = scan_skills(path, false)?;
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

fn print_audit_report(skills: &[Skill]) {
    println!("\n=== Medusa Skill Audit Report (v0.11) ===\n");
    
    for skill in skills {
        println!("Skill: {} ({}), level: {}", skill.label, skill.id, skill.level);
        println!("  Experience: {:.1}/100", skill.experience);
        println!("  Confidence: {:.0}%", skill.confidence * 100.0);
        println!("  Metrics:");
        println!("    - Content Length: {} chars", skill.metrics.content_length);
        println!("    - Code Blocks: {}", skill.metrics.code_blocks);
        println!("    - Step Instructions: {}", skill.metrics.step_count);
        println!("    - Technical Terms: {}", skill.metrics.tech_term_count);
        println!("    - Complexity Score: {:.1}/100", skill.metrics.complexity_score);
        println!("    - Value Score: {:.1}/100", skill.metrics.value_score);
        println!();
    }
    
    // Summary statistics
    let avg_complexity = skills.iter().map(|s| s.metrics.complexity_score).sum::<f64>() / skills.len() as f64;
    let avg_value = skills.iter().map(|s| s.metrics.value_score).sum::<f64>() / skills.len() as f64;
    
    println!("=== Summary ===");
    println!("Total Skills: {}", skills.len());
    println!("Average Complexity: {:.1}/100", avg_complexity);
    println!("Average Value: {:.1}/100", avg_value);
}

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
                eprintln!("Usage: medusa scan <path> [--sequential]");
                return;
            }
            let path = &args[2];
            let sequential = args.iter().any(|a| a == "--sequential");
            match scan_skills(path, !sequential) {
                Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "html" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments");
                eprintln!("Usage: medusa html <path> <output> [--sequential]");
                return;
            }
            let path = &args[2];
            let output = &args[3];
            let sequential = args.iter().any(|a| a == "--sequential");
            match scan_skills(path, !sequential) {
                Ok(result) => {
                    match generate_html(&result, output) {
                        Ok(_) => eprintln!("HTML report generated: {}", output),
                        Err(e) => eprintln!("Error writing HTML: {}", e),
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
        "update" => {
            eprintln!("Updating Medusa from GitHub...");
            match std::process::Command::new("git")
                .args(&["-C", ".", "pull", "https://github.com/jtshow/medusa.git"])
                .status()
            {
                Ok(status) if status.success() => {
                    eprintln!("✅ Pull successful, rebuilding...");
                    match std::process::Command::new("cargo")
                        .args(&["build", "--release"])
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
        "audit" => {
            if args.len() < 3 {
                eprintln!("Error: Missing path argument");
                eprintln!("Usage: medusa audit <path>");
                return;
            }
            match scan_skills(&args[2], true) {
                Ok(result) => print_audit_report(&result.skills),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_help();
        }
    }
}
