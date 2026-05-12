use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub timestamp: String,
    pub skills: Vec<SkillSnapshot>,
    pub total_skills: usize,
    pub avg_experience: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSnapshot {
    pub id: String,
    pub experience: f64,
    pub level: String,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightType {
    RecurringGap,
    Improvement,
    Decline,
    Stable,
    NewSkill,
    ResolvedGap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Fluctuating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamInsight {
    pub id: String,
    pub insight_type: InsightType,
    pub skill_id: String,
    pub description: String,
    pub severity: f64,
    pub first_detected: String,
    pub last_detected: String,
    pub occurrences: usize,
    pub trend: TrendDirection,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamKnowledgeBase {
    pub insights: Vec<DreamInsight>,
    pub last_dream_time: Option<String>,
    pub total_sessions_analyzed: usize,
    pub total_patterns_found: usize,
}

impl DreamKnowledgeBase {
    pub fn new() -> Self {
        DreamKnowledgeBase {
            insights: Vec::new(),
            last_dream_time: None,
            total_sessions_analyzed: 0,
            total_patterns_found: 0,
        }
    }
}

pub fn get_history_path(path: &Path) -> std::path::PathBuf {
    path.join(".medusa_history.json")
}

pub fn get_dream_path(path: &Path) -> std::path::PathBuf {
    path.join(".medusa_dream.json")
}

fn find_dream_files(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = Some(path);
    while let Some(dir) = current {
        let dream_path = dir.join(".medusa_dream.json");
        if dream_path.exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

pub fn load_knowledge_base_from_path(path: &Path) -> DreamKnowledgeBase {
    // First try the exact path
    let dream_path = get_dream_path(path);
    if dream_path.exists() {
        return load_knowledge_base(path);
    }
    // Fall back to walking up the tree
    match find_dream_files(path) {
        Some(found_dir) => load_knowledge_base(&found_dir),
        None => DreamKnowledgeBase::new(),
    }
}

pub fn load_history(path: &Path) -> Vec<SessionRecord> {
    let history_path = get_history_path(path);
    if history_path.exists() {
        fs::read_to_string(&history_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn load_knowledge_base(path: &Path) -> DreamKnowledgeBase {
    let dream_path = get_dream_path(path);
    if dream_path.exists() {
        fs::read_to_string(&dream_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(DreamKnowledgeBase::new)
    } else {
        DreamKnowledgeBase::new()
    }
}

pub fn save_knowledge_base(path: &Path, kb: &DreamKnowledgeBase) {
    let dream_path = get_dream_path(path);
    if let Ok(json) = serde_json::to_string_pretty(kb) {
        let _ = fs::write(&dream_path, json);
    }
}

pub fn save_history(path: &Path, history: &[SessionRecord]) {
    let history_path = get_history_path(path);
    if let Ok(json) = serde_json::to_string_pretty(history) {
        let _ = fs::write(&history_path, json);
    }
}

pub fn from_skill(skill: &super::Skill) -> SkillSnapshot {
    SkillSnapshot {
        id: skill.id.clone(),
        experience: skill.experience,
        level: skill.level.clone(),
        gaps: skill.context.gaps.clone(),
    }
}

pub fn record_session(path: &Path, skills: &[SkillSnapshot]) {
    let mut history = load_history(path);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let total = skills.len();
    let avg = if total > 0 {
        skills.iter().map(|s| s.experience).sum::<f64>() / total as f64
    } else {
        0.0
    };

    history.push(SessionRecord {
        timestamp: now,
        skills: skills.to_vec(),
        total_skills: total,
        avg_experience: avg,
    });

    if history.len() > 50 {
        history.drain(0..history.len() - 50);
    }

    save_history(path, &history);
}

#[allow(dead_code)]
pub fn run_dream(path: &Path) -> DreamKnowledgeBase {
    run_dream_with_config(path, None)
}

pub fn run_dream_with_config(path: &Path, config: Option<&super::DreamingConfig>) -> DreamKnowledgeBase {
    let cfg = config.cloned().unwrap_or_default();
    let history = load_history(path);
    let mut kb = load_knowledge_base(path);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if history.len() < 2 {
        let mut insight = DreamInsight {
            id: "dream-init".to_string(),
            insight_type: InsightType::Stable,
            skill_id: "system".to_string(),
            description: "Not enough sessions to dream. Need at least 2 scan sessions.".to_string(),
            severity: 0.0,
            first_detected: now.clone(),
            last_detected: now.clone(),
            occurrences: 1,
            trend: TrendDirection::Stable,
            metadata: HashMap::new(),
        };
        insight.metadata.insert("sessions_available".to_string(), history.len().to_string());
        kb.insights.push(insight);
        kb.total_sessions_analyzed = history.len();
        save_knowledge_base(path, &kb);
        return kb;
    }

    let mut new_insights: Vec<DreamInsight> = Vec::new();

    // Detect skill-level patterns by comparing consecutive sessions
    let all_skill_ids = collect_all_skill_ids(&history);
    for skill_id in all_skill_ids {
        let appearances = get_skill_appearances(&history, &skill_id);
        if appearances.len() < 2 {
            if appearances.len() == 1 {
                new_insights.push(DreamInsight {
                    id: format!("new-{}", skill_id),
                    insight_type: InsightType::NewSkill,
                    skill_id: skill_id.clone(),
                    description: format!("'{}' appeared for the first time in the latest scan.", skill_id),
                    severity: 0.3,
                    first_detected: appearances[0].0.clone(),
                    last_detected: appearances[0].0.clone(),
                    occurrences: 1,
                    trend: TrendDirection::Stable,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("experience".to_string(), format!("{:.1}", appearances[0].1));
                        m
                    },
                });
            }
            continue;
        }

        let exp_values: Vec<f64> = appearances.iter().map(|(_, e, _)| *e).collect();
        let first_exp = exp_values.first().copied().unwrap_or(0.0);
        let last_exp = exp_values.last().copied().unwrap_or(0.0);
        let diff = last_exp - first_exp;
        let first_ts = appearances.first().map(|(t, _, _)| t.clone()).unwrap_or_else(|| "unknown".to_string());
        let last_ts = appearances.last().map(|(t, _, _)| t.clone()).unwrap_or_else(|| "unknown".to_string());

        // Collect gaps from the most recent session
        let latest_gaps: Vec<String> = appearances.last()
            .map(|(_, _, g)| g.clone())
            .unwrap_or_default();

        // Check for recurring gaps
        let gap_frequency = count_gap_frequency(&appearances);
        for (gap, count) in &gap_frequency {
            if *count >= 3 {
                new_insights.push(DreamInsight {
                    id: format!("gap-{}-{}", skill_id, gap.replace(' ', "-").chars().take(20).collect::<String>()),
                    insight_type: InsightType::RecurringGap,
                    skill_id: skill_id.clone(),
                    description: format!("Recurring gap in '{}': {}. Appeared in {} of {} sessions.", skill_id, gap, count, appearances.len()),
                    severity: (*count as f64 / appearances.len() as f64).min(1.0),
                    first_detected: first_ts.clone(),
                    last_detected: last_ts.clone(),
                    occurrences: *count,
                    trend: TrendDirection::Stable,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("gap".to_string(), gap.clone());
                        m.insert("frequency".to_string(), count.to_string());
                        m
                    },
                });
            }
        }

        // Check if any gaps were resolved
        let earlier_gaps = get_all_historical_gaps(&appearances);
        for gap in &earlier_gaps {
            if !latest_gaps.contains(gap) {
                new_insights.push(DreamInsight {
                    id: format!("resolved-{}-{}", skill_id, gap.replace(' ', "-").chars().take(20).collect::<String>()),
                    insight_type: InsightType::ResolvedGap,
                    skill_id: skill_id.clone(),
                    description: format!("Gap resolved in '{}': {}. No longer present in latest scan.", skill_id, gap),
                    severity: 0.6,
                    first_detected: first_ts.clone(),
                    last_detected: last_ts.clone(),
                    occurrences: 1,
                    trend: TrendDirection::Improving,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("gap".to_string(), gap.clone());
                        m
                    },
                });
            }
        }

        // Trend analysis
        if diff.abs() >= 5.0 {
            if diff > 0.0 {
                new_insights.push(DreamInsight {
                    id: format!("improve-{}", skill_id),
                    insight_type: InsightType::Improvement,
                    skill_id: skill_id.clone(),
                    description: format!("'{}' is improving: {:.1} → {:.1} (+{:.1} pts across {} sessions).", skill_id, first_exp, last_exp, diff, appearances.len()),
                    severity: (diff / 100.0).min(1.0),
                    first_detected: first_ts.clone(),
                    last_detected: last_ts.clone(),
                    occurrences: appearances.len(),
                    trend: TrendDirection::Improving,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("delta".to_string(), format!("{:.1}", diff));
                        m
                    },
                });
            } else {
                new_insights.push(DreamInsight {
                    id: format!("decline-{}", skill_id),
                    insight_type: InsightType::Decline,
                    skill_id: skill_id.clone(),
                    description: format!("'{}' is declining: {:.1} → {:.1} ({:.1} pts across {} sessions).", skill_id, first_exp, last_exp, diff.abs(), appearances.len()),
                    severity: (diff.abs() / 100.0).min(1.0),
                    first_detected: first_ts.clone(),
                    last_detected: last_ts.clone(),
                    occurrences: appearances.len(),
                    trend: TrendDirection::Declining,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("delta".to_string(), format!("{:.1}", diff));
                        m
                    },
                });
            }
        } else {
            let has_stable = appearances.len() >= 3;
            if has_stable {
                new_insights.push(DreamInsight {
                    id: format!("stable-{}", skill_id),
                    insight_type: InsightType::Stable,
                    skill_id: skill_id.clone(),
                    description: format!("'{}' is stable at ~{:.1} across {} sessions (within {} pt range).", skill_id, last_exp, appearances.len(), diff.abs()),
                    severity: 0.2,
                    first_detected: first_ts.clone(),
                    last_detected: last_ts.clone(),
                    occurrences: appearances.len(),
                    trend: TrendDirection::Stable,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("avg_experience".to_string(), format!("{:.1}", exp_values.iter().sum::<f64>() / exp_values.len() as f64));
                        m
                    },
                });
            }
        }
    }

    // Calculate aggregate patterns
    let improving_count = new_insights.iter().filter(|i| i.insight_type == InsightType::Improvement).count();
    let declining_count = new_insights.iter().filter(|i| i.insight_type == InsightType::Decline).count();
    let recurring_gap_count = new_insights.iter().filter(|i| i.insight_type == InsightType::RecurringGap).count();
    let resolved_gap_count = new_insights.iter().filter(|i| i.insight_type == InsightType::ResolvedGap).count();

    if improving_count > 0 || declining_count > 0 {
        new_insights.push(DreamInsight {
            id: "dream-aggregate".to_string(),
            insight_type: InsightType::Stable,
            skill_id: "system".to_string(),
            description: format!("Dream cycle complete: {} improving, {} declining, {} recurring gaps, {} resolved gaps.", improving_count, declining_count, recurring_gap_count, resolved_gap_count),
            severity: if recurring_gap_count > 0 { 0.8 } else { 0.3 },
            first_detected: now.clone(),
            last_detected: now.clone(),
            occurrences: 1,
            trend: if improving_count > declining_count { TrendDirection::Improving } else { TrendDirection::Declining },
            metadata: {
                let mut m = HashMap::new();
                m.insert("improving".to_string(), improving_count.to_string());
                m.insert("declining".to_string(), declining_count.to_string());
                m.insert("recurring_gaps".to_string(), recurring_gap_count.to_string());
                m.insert("resolved_gaps".to_string(), resolved_gap_count.to_string());
                m
            },
        });
    }

    // Merge with existing KB: keep old insights that are still relevant, add new ones
    merge_insights(&mut kb, new_insights, cfg.max_insights);

    // Consolidate: merge duplicates, prune low-value insights
    let report = consolidate_with_config(&mut kb, Some(&cfg));
    if report.total_before > 0 && report.total_after < report.total_before {
        eprintln!("  [dream] Consolidation: {} → {} insights ({} merged, {} pruned)",
            report.total_before, report.total_after, report.duplicate_merged, report.low_severity_pruned + report.pruned_count);
    }

    kb.last_dream_time = Some(now);
    kb.total_sessions_analyzed = history.len();
    kb.total_patterns_found = kb.insights.len();

    save_knowledge_base(path, &kb);
    kb
}

fn collect_all_skill_ids(history: &[SessionRecord]) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for session in history {
        for skill in &session.skills {
            if !ids.contains(&skill.id) {
                ids.push(skill.id.clone());
            }
        }
    }
    ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub total_before: usize,
    pub total_after: usize,
    pub merged_count: usize,
    pub pruned_count: usize,
    pub low_severity_pruned: usize,
    pub duplicate_merged: usize,
}

#[allow(dead_code)]
pub fn consolidate(kb: &mut DreamKnowledgeBase) -> ConsolidationReport {
    consolidate_with_config(kb, None)
}

pub fn consolidate_with_config(kb: &mut DreamKnowledgeBase, config: Option<&super::DreamingConfig>) -> ConsolidationReport {
    let cfg = config.cloned().unwrap_or_default();
    let before = kb.insights.len();
    let mut merged = 0usize;
    let mut pruned = 0usize;
    let mut low_pri_pruned = 0usize;

    // Phase 1: Merge duplicates by skill_id + insight_type + gap metadata key
    let mut merged_insights: Vec<DreamInsight> = Vec::new();
    let mut used: Vec<bool> = vec![false; kb.insights.len()];

    for i in 0..kb.insights.len() {
        if used[i] { continue; }
        let mut base = kb.insights[i].clone();
        used[i] = true;

        for (j, other) in kb.insights.iter().enumerate().skip(i + 1) {

            let same_type = base.insight_type == other.insight_type;
            let same_skill = base.skill_id == other.skill_id;
            let same_gap = matches!(base.insight_type, InsightType::RecurringGap | InsightType::ResolvedGap)
                && same_skill
                && base.metadata.get("gap") == other.metadata.get("gap");
            let is_trend_same = same_skill
                && matches!(base.insight_type, InsightType::Improvement | InsightType::Decline | InsightType::Stable)
                && base.insight_type == other.insight_type;

            if same_type && (same_gap || is_trend_same) {
                base.occurrences = base.occurrences.max(other.occurrences);
                base.severity = (base.severity + other.severity) / 2.0;
                if other.last_detected > base.last_detected {
                    base.last_detected = other.last_detected.clone();
                }
                used[j] = true;
                merged += 1;
            }
        }
        merged_insights.push(base);
    }

    // Phase 2: Prune low-severity stable insights
    let stable_threshold = 0.15 * (1.0 - (cfg.retention_percent - 0.5).max(0.0) * 2.0);
    merged_insights.retain(|insight| {
        if insight.insight_type == InsightType::Stable && insight.severity < stable_threshold {
            low_pri_pruned += 1;
            return false;
        }
        true
    });

    merged_insights.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

    let max_insights = cfg.max_insights;
    let before_prune = merged_insights.len();
    if merged_insights.len() > max_insights {
        merged_insights.truncate(max_insights);
        pruned = before_prune - merged_insights.len();
    }

    let after = merged_insights.len();
    kb.insights = merged_insights;
    kb.total_patterns_found = after;

    ConsolidationReport {
        total_before: before,
        total_after: after,
        merged_count: merged,
        pruned_count: pruned,
        low_severity_pruned: low_pri_pruned,
        duplicate_merged: merged,
    }
}

pub fn get_insights_for_skill<'a>(kb: &'a DreamKnowledgeBase, skill_id: &str) -> Vec<&'a DreamInsight> {
    kb.insights.iter().filter(|i| i.skill_id == skill_id).collect()
}

pub fn get_cross_session_summary(kb: &DreamKnowledgeBase, skill_id: &str) -> Vec<String> {
    let mut summary = Vec::new();
    let insights = get_insights_for_skill(kb, skill_id);

    for insight in &insights {
        match insight.insight_type {
            InsightType::RecurringGap => {
                summary.push(format!("[Recurring] {} (seen in {} of {} sessions)", 
                    insight.metadata.get("gap").map(|s| s.as_str()).unwrap_or("unknown gap"),
                    insight.occurrences, kb.total_sessions_analyzed));
            }
            InsightType::Improvement => {
                summary.push(format!("[Improving] +{} pts over {} sessions",
                    insight.metadata.get("delta").map(|s| s.as_str()).unwrap_or("?"),
                    insight.occurrences));
            }
            InsightType::Decline => {
                summary.push(format!("[Declining] {} pts over {} sessions",
                    insight.metadata.get("delta").map(|s| s.as_str()).unwrap_or("?"),
                    insight.occurrences));
            }
            InsightType::ResolvedGap => {
                summary.push(format!("[Resolved] Gap '{}' no longer present",
                    insight.metadata.get("gap").map(|s| s.as_str()).unwrap_or("unknown")));
            }
            InsightType::NewSkill => {
                summary.push("[New] First appearance in recent scans".to_string());
            }
            InsightType::Stable => {
                summary.push(format!("[Stable] ~{} across {} sessions",
                    insight.metadata.get("avg_experience").map(|s| s.as_str()).unwrap_or("?"),
                    insight.occurrences));
            }
        }
    }
    summary
}

pub fn print_consolidation_report(report: &ConsolidationReport) {
    if report.total_before == report.total_after {
        return;
    }
    println!("\n  === Consolidation ===");
    println!("  Before: {} insights", report.total_before);
    println!("  After: {} insights", report.total_after);
    if report.duplicate_merged > 0 {
        println!("  Merged: {} duplicates", report.duplicate_merged);
    }
    if report.low_severity_pruned > 0 {
        println!("  Pruned: {} low-severity stable insights", report.low_severity_pruned);
    }
    if report.pruned_count > 0 {
        println!("  Truncated: {} beyond max limit", report.pruned_count);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSuggestion {
    pub gap_pattern: String,
    pub suggestion: String,
    pub expected_impact: String,
    pub effort: String,
    pub category: String,
}

pub fn get_builtin_suggestions() -> Vec<LearningSuggestion> {
    vec![
        LearningSuggestion {
            gap_pattern: "code examples".to_string(),
            suggestion: "Add code blocks (```...```) demonstrating practical usage. Each block adds +5 complexity (max 25 pts).".to_string(),
            expected_impact: "Up to +25 points to complexity score".to_string(),
            effort: "medium".to_string(),
            category: "content".to_string(),
        },
        LearningSuggestion {
            gap_pattern: "step-by-step".to_string(),
            suggestion: "Add numbered steps or bulleted instructions. Each step adds +2 complexity (max 20 pts). Target 10+ steps.".to_string(),
            expected_impact: "Up to +20 points to complexity score".to_string(),
            effort: "low".to_string(),
            category: "structure".to_string(),
        },
        LearningSuggestion {
            gap_pattern: "technical terms".to_string(),
            suggestion: "Incorporate terms from: algorithm, implementation, architecture, framework, optimization, scalability, security, encryption, authentication, database, API, SDK, middleware.".to_string(),
            expected_impact: "Up to +25 points to complexity score".to_string(),
            effort: "low".to_string(),
            category: "language".to_string(),
        },
        LearningSuggestion {
            gap_pattern: "content".to_string(),
            suggestion: "Expand the skill documentation to 3000+ characters. Every 100 chars = +1 complexity (max 30 pts).".to_string(),
            expected_impact: "Up to +30 points to complexity score".to_string(),
            effort: "medium".to_string(),
            category: "content".to_string(),
        },
        LearningSuggestion {
            gap_pattern: "all components".to_string(),
            suggestion: "Include all three: code blocks (>0), step instructions (>5), AND technical terms (>3) for the +10 bonus.".to_string(),
            expected_impact: "+10 bonus points to complexity score".to_string(),
            effort: "medium".to_string(),
            category: "content".to_string(),
        },
        LearningSuggestion {
            gap_pattern: "keyword".to_string(),
            suggestion: "Add relevant keywords to the description: advanced, expert, senior, security, rust, python, kubernetes, etc.".to_string(),
            expected_impact: "+2-8 points per keyword (10% weight)".to_string(),
            effort: "low".to_string(),
            category: "metadata".to_string(),
        },
        LearningSuggestion {
            gap_pattern: "fusion".to_string(),
            suggestion: "Combine related skills into unified documentation to reduce fragmentation and increase effective complexity.".to_string(),
            expected_impact: "Improves overall skill tree coherence".to_string(),
            effort: "high".to_string(),
            category: "strategy".to_string(),
        },
    ]
}

pub fn get_suggestions_for_gap(gap: &str) -> Vec<LearningSuggestion> {
    let gap_lower = gap.to_lowercase();
    get_builtin_suggestions().into_iter().filter(|s| {
        gap_lower.contains(&s.gap_pattern.to_lowercase())
    }).collect()
}

pub fn get_learning_path_for_skill(skill_id: &str, kb: &DreamKnowledgeBase) -> Vec<String> {
    let mut path = Vec::new();
    let insights = get_insights_for_skill(kb, skill_id);
    let mut seen_gaps: Vec<String> = Vec::new();

    for insight in &insights {
        if insight.insight_type == InsightType::RecurringGap {
            if let Some(gap) = insight.metadata.get("gap") {
                if !seen_gaps.contains(gap) {
                    seen_gaps.push(gap.clone());
                    let suggestions = get_suggestions_for_gap(gap);
                    path.push(format!("  Gap: {}", gap));
                    for s in suggestions.iter() {
                        path.push(format!("    → {} [Impact: {}]", s.suggestion, s.expected_impact));
                    }
                }
            }
        }
    }

    if path.is_empty() {
        path.push(format!("  No recurring gaps found for '{}'. Skill appears healthy.", skill_id));
    }

    path
}

pub fn print_learning_path(skill_id: &str, path_lines: &[String]) {
    println!("\n=== Learning Path: {} ===", skill_id);
    for line in path_lines {
        println!("{}", line);
    }
}

fn get_skill_appearances(history: &[SessionRecord], skill_id: &str) -> Vec<(String, f64, Vec<String>)> {
    history.iter().filter_map(|session| {
        session.skills.iter().find(|s| s.id == skill_id).map(|s| {
            (session.timestamp.clone(), s.experience, s.gaps.clone())
        })
    }).collect()
}

fn count_gap_frequency(appearances: &[(String, f64, Vec<String>)]) -> HashMap<String, usize> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for (_, _, gaps) in appearances {
        for gap in gaps {
            *freq.entry(gap.clone()).or_insert(0) += 1;
        }
    }
    freq
}

fn get_all_historical_gaps(appearances: &[(String, f64, Vec<String>)]) -> Vec<String> {
    let mut gaps: Vec<String> = Vec::new();
    for (_, _, skill_gaps) in appearances.iter().rev().skip(1) {
        for g in skill_gaps {
            if !gaps.contains(g) {
                gaps.push(g.clone());
            }
        }
    }
    gaps
}

fn merge_insights(kb: &mut DreamKnowledgeBase, new_insights: Vec<DreamInsight>, max_insights: usize) {
    for new_i in new_insights {
        let existing = kb.insights.iter_mut().find(|i| i.id == new_i.id);
        if let Some(old) = existing {
            old.last_detected = new_i.last_detected;
            old.occurrences = new_i.occurrences;
            old.severity = new_i.severity;
            old.trend = new_i.trend;
            old.metadata = new_i.metadata;
        } else {
            kb.insights.push(new_i);
        }
    }
    kb.insights.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
    if kb.insights.len() > max_insights {
        kb.insights.truncate(max_insights);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamDiary {
    pub session_count: usize,
    pub date_range: String,
    pub avg_experience_trend: String,
    pub skill_entries: Vec<SkillDiaryEntry>,
    pub pattern_summary: PatternDiarySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDiaryEntry {
    pub skill_id: String,
    pub timeline: Vec<(String, f64)>, // (timestamp, experience)
    pub gaps_over_time: Vec<(String, Vec<String>)>, // (timestamp, gaps)
    pub trend_direction: String,
    pub gap_count_change: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDiarySummary {
    pub recurring_gaps: Vec<String>,
    pub resolved_gaps: Vec<String>,
    pub improvements: Vec<String>,
    pub declines: Vec<String>,
    pub new_skills: Vec<String>,
}

pub fn generate_dream_diary(path: &Path) -> DreamDiary {
    let history = load_history(path);
    let kb = load_knowledge_base_from_path(path);

    let session_count = history.len();

    // Date range
    let date_range = if session_count >= 2 {
        format!("{} → {}",
            history.first().map(|s| &s.timestamp[..10]).unwrap_or("?"),
            history.last().map(|s| &s.timestamp[..10]).unwrap_or("?"))
    } else if session_count == 1 {
        history[0].timestamp[..10].to_string()
    } else {
        "No sessions".to_string()
    };

    // Average experience trend
    let avg_exp_trend = if session_count >= 2 {
        let first_avg = history.first().map(|s| s.avg_experience).unwrap_or(0.0);
        let last_avg = history.last().map(|s| s.avg_experience).unwrap_or(0.0);
        let diff = last_avg - first_avg;
        if diff.abs() < 0.5 {
            format!("Stable ~{:.1} across {} sessions", last_avg, session_count)
        } else if diff > 0.0 {
            format!("Improved from {:.1} → {:.1} (+{:.1} across {} sessions)", first_avg, last_avg, diff, session_count)
        } else {
            format!("Declined from {:.1} → {:.1} ({:.1} across {} sessions)", first_avg, last_avg, diff.abs(), session_count)
        }
    } else if session_count == 1 {
        format!("{:.1} (first session)", history[0].avg_experience)
    } else {
        "N/A".to_string()
    };

    // Per-skill timelines
    let mut skill_entries = Vec::new();
    let all_ids = collect_all_skill_ids(&history);
    for skill_id in &all_ids {
        let appearances = get_skill_appearances(&history, skill_id);
        if appearances.is_empty() { continue; }

        let timeline: Vec<(String, f64)> = appearances.iter()
            .map(|(ts, exp, _)| (ts[..10].to_string(), *exp))
            .collect();

        let gaps_over_time: Vec<(String, Vec<String>)> = appearances.iter()
            .map(|(ts, _, gaps)| (ts[..10].to_string(), gaps.clone()))
            .collect();

        // Trend direction
        let first_exp = timeline.first().map(|(_, e)| *e).unwrap_or(0.0);
        let last_exp = timeline.last().map(|(_, e)| *e).unwrap_or(0.0);
        let diff = last_exp - first_exp;
        let trend_direction = if diff.abs() < 1.0 { "Stable".to_string()
            } else if diff > 0.0 { format!("Improving (+{:.1})", diff)
            } else { format!("Declining ({:.1})", diff.abs()) };

        // Gap count change
        let first_gap_count = gaps_over_time.first().map(|(_, g)| g.len()).unwrap_or(0);
        let last_gap_count = gaps_over_time.last().map(|(_, g)| g.len()).unwrap_or(0);
        let gap_count_change = if first_gap_count == last_gap_count {
            format!("{} gaps (unchanged)", first_gap_count)
        } else if last_gap_count < first_gap_count {
            format!("{} → {} gaps ({} resolved)", first_gap_count, last_gap_count, first_gap_count - last_gap_count)
        } else {
            format!("{} → {} gaps ({} new)", first_gap_count, last_gap_count, last_gap_count - first_gap_count)
        };

        skill_entries.push(SkillDiaryEntry { skill_id: skill_id.clone(), timeline, gaps_over_time, trend_direction, gap_count_change });
    }

    // Pattern summary from KB
    let recurring_gaps: Vec<String> = kb.insights.iter()
        .filter(|i| i.insight_type == InsightType::RecurringGap)
        .map(|i| i.description.clone())
        .collect();
    let resolved_gaps: Vec<String> = kb.insights.iter()
        .filter(|i| i.insight_type == InsightType::ResolvedGap)
        .map(|i| i.description.clone())
        .collect();
    let improvements: Vec<String> = kb.insights.iter()
        .filter(|i| i.insight_type == InsightType::Improvement)
        .map(|i| i.description.clone())
        .collect();
    let declines: Vec<String> = kb.insights.iter()
        .filter(|i| i.insight_type == InsightType::Decline)
        .map(|i| i.description.clone())
        .collect();
    let new_skills: Vec<String> = kb.insights.iter()
        .filter(|i| i.insight_type == InsightType::NewSkill)
        .map(|i| i.description.clone())
        .collect();

    DreamDiary {
        session_count,
        date_range,
        avg_experience_trend: avg_exp_trend,
        skill_entries,
        pattern_summary: PatternDiarySummary { recurring_gaps, resolved_gaps, improvements, declines, new_skills },
    }
}

pub fn print_dream_diary(diary: &DreamDiary) {
    println!("\n=== Medusa Dream Diary ===");
    println!("Sessions: {} | Date Range: {}", diary.session_count, diary.date_range);
    println!("Experience Trend: {}", diary.avg_experience_trend);

    if diary.skill_entries.is_empty() {
        println!("\nNo skills recorded yet. Run scans to build a dream diary.");
        return;
    }

    println!("\n--- Skill Evolution ---");
    for entry in &diary.skill_entries {
        println!("\n  {} — {} | {}", entry.skill_id, entry.trend_direction, entry.gap_count_change);
        let bar_width = 20usize;
        if entry.timeline.len() >= 2 {
            let min_exp = entry.timeline.iter().map(|(_, e)| *e).fold(f64::MAX, |a, b| a.min(b));
            let max_exp = entry.timeline.iter().map(|(_, e)| *e).fold(f64::MIN, |a, b| a.max(b));
            let range = (max_exp - min_exp).max(1.0);
            print!("    Timeline:");
            for (ts, exp) in &entry.timeline {
                let pos = ((exp - min_exp) / range * bar_width as f64).round() as usize;
                let pos = pos.min(bar_width);
                print!(" {}|{}{}", &ts[5..], "█".repeat(pos), "░".repeat(bar_width - pos));
            }
            println!();
        } else {
            println!("    Single session at {:.1}", entry.timeline.first().map(|(_, e)| *e).unwrap_or(0.0));
        }

        // Show gap evolution
        let gap_snapshots: Vec<&(String, Vec<String>)> = entry.gaps_over_time.iter().collect();
        if gap_snapshots.len() >= 2 {
            for i in 1..gap_snapshots.len() {
                let prev_gaps = &gap_snapshots[i - 1].1;
                let curr_gaps = &gap_snapshots[i].1;
                let new_gaps: Vec<&String> = curr_gaps.iter().filter(|g| !prev_gaps.contains(g)).collect();
                let gone_gaps: Vec<&String> = prev_gaps.iter().filter(|g| !curr_gaps.contains(g)).collect();
                if !new_gaps.is_empty() {
                    println!("    [+{}] {}", gap_snapshots[i].0, new_gaps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                }
                if !gone_gaps.is_empty() {
                    println!("    [-{}] {}", gap_snapshots[i].0, gone_gaps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                }
            }
        }
    }

    let ps = &diary.pattern_summary;
    if !ps.recurring_gaps.is_empty() {
        println!("\n--- Recurring Gaps ---");
        for g in ps.recurring_gaps.iter().take(5) {
            println!("  • {}", g);
        }
    }
    if !ps.resolved_gaps.is_empty() {
        println!("\n--- Resolved Gaps ---");
        for g in ps.resolved_gaps.iter().take(5) {
            println!("  ✓ {}", g);
        }
    }
    if !ps.improvements.is_empty() {
        println!("\n--- Improvements ---");
        for g in ps.improvements.iter().take(5) {
            println!("  ↑ {}", g);
        }
    }
    if !ps.declines.is_empty() {
        println!("\n--- Declines ---");
        for g in ps.declines.iter().take(5) {
            println!("  ↓ {}", g);
        }
    }
    if !ps.new_skills.is_empty() {
        println!("\n--- New Skills ---");
        for g in ps.new_skills.iter().take(5) {
            println!("  ✦ {}", g);
        }
    }
    println!();
}

pub fn export_dream_diary_md(diary: &DreamDiary) -> String {
    let mut md = String::new();
    md.push_str("# Medusa Dream Diary\n\n");
    md.push_str(&format!("- **Sessions**: {} | **Date Range**: {}\n", diary.session_count, diary.date_range));
    md.push_str(&format!("- **Experience Trend**: {}\n\n", diary.avg_experience_trend));

    if diary.skill_entries.is_empty() {
        md.push_str("No skills recorded yet.\n");
        return md;
    }

    md.push_str("## Skill Evolution\n\n");
    for entry in &diary.skill_entries {
        md.push_str(&format!("### {}\n", entry.skill_id));
        md.push_str(&format!("- **Trend**: {} | **Gaps**: {}\n", entry.trend_direction, entry.gap_count_change));
        md.push_str("| Date | Experience |\n|------|-----------|\n");
        for (ts, exp) in &entry.timeline {
            md.push_str(&format!("| {} | {:.1} |\n", ts, exp));
        }
        md.push('\n');

        // Gap evolution
        let gap_snapshots: Vec<&(String, Vec<String>)> = entry.gaps_over_time.iter().collect();
        if gap_snapshots.len() >= 2 {
            md.push_str("**Gap Changes:**\n");
            for i in 1..gap_snapshots.len() {
                let prev_gaps = &gap_snapshots[i - 1].1;
                let curr_gaps = &gap_snapshots[i].1;
                let new_gaps: Vec<&String> = curr_gaps.iter().filter(|g| !prev_gaps.contains(g)).collect();
                let gone_gaps: Vec<&String> = prev_gaps.iter().filter(|g| !curr_gaps.contains(g)).collect();
                if !new_gaps.is_empty() {
                    md.push_str(&format!("- **New** ({}): {}\n", gap_snapshots[i].0, new_gaps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                }
                if !gone_gaps.is_empty() {
                    md.push_str(&format!("- **Resolved** ({}): {}\n", gap_snapshots[i].0, gone_gaps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                }
            }
            md.push('\n');
        }
    }

    let ps = &diary.pattern_summary;
    if !ps.recurring_gaps.is_empty() {
        md.push_str("## Recurring Gaps\n\n");
for g in &ps.recurring_gaps { md.push_str(&format!("- {}\n", g)); }
        md.push('\n');
    }
    if !ps.resolved_gaps.is_empty() {
        for g in &ps.resolved_gaps { md.push_str(&format!("- ✓ {}\n", g)); }
        md.push('\n');
    }
    if !ps.improvements.is_empty() {
        for g in &ps.improvements { md.push_str(&format!("- ↑ {}\n", g)); }
        md.push('\n');
    }
    if !ps.declines.is_empty() {
        for g in &ps.declines { md.push_str(&format!("- ↓ {}\n", g)); }
        md.push('\n');
    }
    if !ps.new_skills.is_empty() {
        for g in &ps.new_skills { md.push_str(&format!("- ✦ {}\n", g)); }
        md.push('\n');
    }
    if !ps.resolved_gaps.is_empty() {
        md.push_str("## Resolved Gaps\n\n");
        for g in &ps.resolved_gaps { md.push_str(&format!("- ✓ {}\n", g)); }
        md.push('\n');
    }
    if !ps.improvements.is_empty() {
        md.push_str("## Improvements\n\n");
        for g in &ps.improvements { md.push_str(&format!("- ↑ {}\n", g)); }
        md.push('\n');
    }
    if !ps.declines.is_empty() {
        md.push_str("## Declines\n\n");
        for g in &ps.declines { md.push_str(&format!("- ↓ {}\n", g)); }
        md.push('\n');
    }
    if !ps.new_skills.is_empty() {
        md.push_str("## New Skills\n\n");
        for g in &ps.new_skills { md.push_str(&format!("- ✦ {}\n", g)); }
        md.push('\n');
    }

    md
}

pub fn print_dream_report(kb: &DreamKnowledgeBase) {
    println!("\n=== Medusa Dream Report ===");
    if let Some(ref t) = kb.last_dream_time {
        println!("Last Dream: {}", t);
    }
    println!("Sessions Analyzed: {}", kb.total_sessions_analyzed);
    println!("Total Patterns: {}", kb.total_patterns_found);

    if kb.insights.is_empty() {
        if kb.total_sessions_analyzed < 2 {
            println!("No sessions recorded yet. Run a scan first: medusa scan <path>");
        } else if kb.total_sessions_analyzed < 3 {
            println!("Only {} session(s) recorded. Need at least 3 identical scan sessions to detect stable/recurring patterns.", kb.total_sessions_analyzed);
            println!("Run 1-2 more scans with: medusa scan <path>");
        } else {
            println!("No patterns detected across {} sessions. Skills may be fluctuating too much for stable pattern detection.", kb.total_sessions_analyzed);
        }
        return;
    }

    let recurring: Vec<&DreamInsight> = kb.insights.iter().filter(|i| i.insight_type == InsightType::RecurringGap).collect();
    let improvements: Vec<&DreamInsight> = kb.insights.iter().filter(|i| i.insight_type == InsightType::Improvement).collect();
    let declines: Vec<&DreamInsight> = kb.insights.iter().filter(|i| i.insight_type == InsightType::Decline).collect();
    let resolved: Vec<&DreamInsight> = kb.insights.iter().filter(|i| i.insight_type == InsightType::ResolvedGap).collect();
    let new_skills: Vec<&DreamInsight> = kb.insights.iter().filter(|i| i.insight_type == InsightType::NewSkill).collect();
    let stable: Vec<&DreamInsight> = kb.insights.iter().filter(|i| i.insight_type == InsightType::Stable && i.skill_id != "system").collect();

    if !recurring.is_empty() {
        println!("\n  🔄 Recurring Gaps (High Priority):");
        for insight in recurring.iter().take(5) {
            println!("    - {} (severity: {:.0}%, {} occurrences)", insight.description, insight.severity * 100.0, insight.occurrences);
            if let Some(gap) = insight.metadata.get("gap") {
                let suggestions = get_suggestions_for_gap(gap);
                for s in suggestions.iter().take(2) {
                    println!("      Suggested fix: {}", s.suggestion);
                }
            }
        }
        println!("\n  Run 'medusa learning-path <path> <skill_id>' for detailed suggestions.");
    }

    if !improvements.is_empty() {
        println!("\n  📈 Improvements:");
        for insight in improvements.iter().take(5) {
            println!("    - {}", insight.description);
        }
    }

    if !declines.is_empty() {
        println!("\n  📉 Declines:");
        for insight in declines.iter().take(5) {
            println!("    - {}", insight.description);
        }
    }

    if !resolved.is_empty() {
        println!("\n  ✅ Resolved Gaps:");
        for insight in resolved.iter().take(5) {
            println!("    - {}", insight.description);
        }
    }

    if !new_skills.is_empty() {
        println!("\n  🆕 New Skills Detected:");
        for insight in new_skills.iter().take(5) {
            println!("    - {}", insight.description);
        }
    }

    if !stable.is_empty() {
        println!("\n  ⚖️  Stable Skills:");
        for insight in stable.iter().take(3) {
            println!("    - {}", insight.description);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SkillMetrics;
    use crate::SkillContext;

    fn sample_history() -> Vec<SessionRecord> {
        vec![
            SessionRecord {
                timestamp: "2026-01-01 10:00:00".to_string(),
                skills: vec![
                    SkillSnapshot { id: "skill-a".to_string(), experience: 30.0, level: "Common".to_string(), gaps: vec!["needs steps".to_string()] },
                    SkillSnapshot { id: "skill-b".to_string(), experience: 50.0, level: "Epic".to_string(), gaps: vec![] },
                ],
                total_skills: 2,
                avg_experience: 40.0,
            },
            SessionRecord {
                timestamp: "2026-01-02 10:00:00".to_string(),
                skills: vec![
                    SkillSnapshot { id: "skill-a".to_string(), experience: 45.0, level: "Mythic".to_string(), gaps: vec!["needs steps".to_string()] },
                    SkillSnapshot { id: "skill-b".to_string(), experience: 55.0, level: "Epic".to_string(), gaps: vec![] },
                ],
                total_skills: 2,
                avg_experience: 50.0,
            },
            SessionRecord {
                timestamp: "2026-01-03 10:00:00".to_string(),
                skills: vec![
                    SkillSnapshot { id: "skill-a".to_string(), experience: 60.0, level: "Epic".to_string(), gaps: vec![] },
                    SkillSnapshot { id: "skill-b".to_string(), experience: 40.0, level: "Mythic".to_string(), gaps: vec!["missing examples".to_string()] },
                    SkillSnapshot { id: "skill-c".to_string(), experience: 10.0, level: "Poor".to_string(), gaps: vec!["too short".to_string()] },
                ],
                total_skills: 3,
                avg_experience: 36.67,
            },
        ]
    }

    #[test]
    fn test_dream_knowledge_base_new() {
        let kb = DreamKnowledgeBase::new();
        assert!(kb.insights.is_empty());
        assert!(kb.last_dream_time.is_none());
        assert_eq!(kb.total_sessions_analyzed, 0);
        assert_eq!(kb.total_patterns_found, 0);
    }

    #[test]
    fn test_from_skill() {
        let skill = crate::Skill {
            id: "test-skill".to_string(),
            label: "Test".to_string(),
            description: "desc".to_string(),
            experience: 50.0,
            level: "Epic".to_string(),
            confidence: 0.8,
            metrics: SkillMetrics { content_length: 100, code_blocks: 2, step_count: 5, tech_term_count: 5, complexity_score: 50.0, value_score: 60.0 },
            context: SkillContext::default(),
        };
        let snapshot = from_skill(&skill);
        assert_eq!(snapshot.id, "test-skill");
        assert_eq!(snapshot.experience, 50.0);
        assert_eq!(snapshot.level, "Epic");
    }

    #[test]
    fn test_collect_all_skill_ids() {
        let history = sample_history();
        let ids = collect_all_skill_ids(&history);
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"skill-a".to_string()));
        assert!(ids.contains(&"skill-b".to_string()));
        assert!(ids.contains(&"skill-c".to_string()));
    }

    #[test]
    fn test_get_skill_appearances() {
        let history = sample_history();
        let appearances = get_skill_appearances(&history, "skill-a");
        assert_eq!(appearances.len(), 3);
        assert_eq!(appearances[0].1, 30.0);
        assert_eq!(appearances[2].1, 60.0);
    }

    #[test]
    fn test_get_skill_appearances_not_found() {
        let history = sample_history();
        let appearances = get_skill_appearances(&history, "nonexistent");
        assert!(appearances.is_empty());
    }

    #[test]
    fn test_count_gap_frequency() {
        let history = sample_history();
        let appearances = get_skill_appearances(&history, "skill-a");
        let freq = count_gap_frequency(&appearances);
        assert_eq!(freq.get("needs steps"), Some(&2)); // appeared in 2 of 3 sessions
    }

    #[test]
    fn test_get_all_historical_gaps() {
        let history = sample_history();
        let appearances = get_skill_appearances(&history, "skill-a");
        let gaps = get_all_historical_gaps(&appearances);
        // The last session has no gaps for skill-a, earlier sessions have "needs steps"
        assert!(gaps.contains(&"needs steps".to_string()));
    }

    #[test]
    fn test_consolidate_with_config() {
        let mut kb = DreamKnowledgeBase::new();
        kb.insights.push(DreamInsight {
            id: "insight-1".to_string(),
            insight_type: InsightType::Improvement,
            skill_id: "skill-a".to_string(),
            description: "Improving".to_string(),
            severity: 0.8,
            first_detected: "2026-01-01".to_string(),
            last_detected: "2026-01-03".to_string(),
            occurrences: 3,
            trend: TrendDirection::Improving,
            metadata: HashMap::new(),
        });
        kb.insights.push(DreamInsight {
            id: "insight-2".to_string(),
            insight_type: InsightType::Stable,
            skill_id: "skill-b".to_string(),
            description: "Stable low severity".to_string(),
            severity: 0.05,
            first_detected: "2026-01-01".to_string(),
            last_detected: "2026-01-03".to_string(),
            occurrences: 3,
            trend: TrendDirection::Stable,
            metadata: HashMap::new(),
        });

        // Use default config since DreamingConfig is private to main.rs
        let report = consolidate_with_config(&mut kb, None);
        assert_eq!(report.total_before, 2);
        // The low-severity stable insight should be pruned
        assert_eq!(report.low_severity_pruned, 1);
        assert_eq!(kb.insights.len(), 1);
    }

    #[test]
    fn test_get_insights_for_skill() {
        let mut kb = DreamKnowledgeBase::new();
        kb.insights.push(DreamInsight {
            id: "1".to_string(),
            insight_type: InsightType::RecurringGap,
            skill_id: "skill-a".to_string(),
            description: "Gap in skill-a".to_string(),
            severity: 0.5,
            first_detected: "2026-01-01".to_string(),
            last_detected: "2026-01-03".to_string(),
            occurrences: 3,
            trend: TrendDirection::Stable,
            metadata: HashMap::new(),
        });
        kb.insights.push(DreamInsight {
            id: "2".to_string(),
            insight_type: InsightType::Improvement,
            skill_id: "skill-b".to_string(),
            description: "Improvement in skill-b".to_string(),
            severity: 0.3,
            first_detected: "2026-01-01".to_string(),
            last_detected: "2026-01-03".to_string(),
            occurrences: 3,
            trend: TrendDirection::Improving,
            metadata: HashMap::new(),
        });

        let insights = get_insights_for_skill(&kb, "skill-a");
        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].id, "1");
    }

    #[test]
    fn test_get_cross_session_summary() {
        let mut kb = DreamKnowledgeBase::new();
        kb.insights.push(DreamInsight {
            id: "1".to_string(),
            insight_type: InsightType::RecurringGap,
            skill_id: "skill-a".to_string(),
            description: "Recurring gap".to_string(),
            severity: 0.5,
            first_detected: "2026-01-01".to_string(),
            last_detected: "2026-01-03".to_string(),
            occurrences: 3,
            trend: TrendDirection::Stable,
            metadata: {
                let mut m = HashMap::new();
                m.insert("gap".to_string(), "needs examples".to_string());
                m
            },
        });

        let summary = get_cross_session_summary(&kb, "skill-a");
        assert!(!summary.is_empty());
        assert!(summary[0].contains("Recurring"));
    }

    #[test]
    fn test_learning_suggestions() {
        let suggestions = get_builtin_suggestions();
        assert!(!suggestions.is_empty());

        let code_suggestions = get_suggestions_for_gap("needs more code examples");
        assert!(!code_suggestions.is_empty());
        assert!(code_suggestions.iter().any(|s| s.gap_pattern == "code examples"));

        let no_suggestions = get_suggestions_for_gap("totally unrelated gap");
        assert!(no_suggestions.is_empty());
    }

    #[test]
    fn test_insight_type_equality() {
        assert_eq!(InsightType::Improvement, InsightType::Improvement);
        assert_ne!(InsightType::Improvement, InsightType::Decline);
    }

    #[test]
    fn test_trend_direction_equality() {
        assert_eq!(TrendDirection::Improving, TrendDirection::Improving);
        assert_ne!(TrendDirection::Improving, TrendDirection::Declining);
    }

    #[test]
    fn test_print_functions_do_not_panic() {
        let _insight = DreamInsight {
            id: "test".to_string(),
            insight_type: InsightType::Improvement,
            skill_id: "skill-a".to_string(),
            description: "Test".to_string(),
            severity: 0.5,
            first_detected: "2026-01-01".to_string(),
            last_detected: "2026-01-03".to_string(),
            occurrences: 3,
            trend: TrendDirection::Improving,
            metadata: HashMap::new(),
        };
        let report = ConsolidationReport {
            total_before: 5,
            total_after: 3,
            merged_count: 1,
            pruned_count: 1,
            low_severity_pruned: 0,
            duplicate_merged: 1,
        };

        print_consolidation_report(&report);

        let kb = DreamKnowledgeBase::new();
        print_dream_report(&kb);
    }
}
