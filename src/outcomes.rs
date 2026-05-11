use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRubric {
    pub skill_id: String,
    pub criteria: Vec<OutcomeCriterion>,
    pub levels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeCriterion {
    pub name: String,
    pub description: String,
    pub good_threshold: f64,
    pub needs_improvement_threshold: f64,
    pub metric_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeAssessment {
    pub skill_id: String,
    pub rubric_name: String,
    pub level: String,
    pub score: f64,
    pub criterion_results: Vec<CriterionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    pub name: String,
    pub actual_value: f64,
    pub threshold_good: f64,
    pub threshold_needs_improvement: f64,
    pub status: CriterionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriterionStatus {
    Good,
    NeedsImprovement,
    Poor,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutcomeStore {
    pub rubrics: HashMap<String, OutcomeRubric>,
}

pub fn get_outcomes_path(path: &Path) -> std::path::PathBuf {
    path.join(".medusa_outcomes.json")
}

fn find_outcomes_file(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = Some(path);
    while let Some(dir) = current {
        let outcomes_path = dir.join(".medusa_outcomes.json");
        if outcomes_path.exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

pub fn load_outcomes(path: &Path) -> OutcomeStore {
    let outcomes_path = get_outcomes_path(path);
    let load_path = if outcomes_path.exists() {
        outcomes_path
    } else {
        match find_outcomes_file(path) {
            Some(found_dir) => found_dir.join(".medusa_outcomes.json"),
            None => return OutcomeStore::default(),
        }
    };
    fs::read_to_string(&load_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_outcomes(path: &Path, store: &OutcomeStore) {
    let outcomes_path = get_outcomes_path(path);
    if let Ok(json) = serde_json::to_string_pretty(store) {
        let _ = fs::write(&outcomes_path, json);
    }
}

pub fn add_rubric(path: &Path, rubric: OutcomeRubric) {
    let mut store = load_outcomes(path);
    store.rubrics.insert(rubric.skill_id.clone(), rubric);
    save_outcomes(path, &store);
}

pub fn remove_rubric(path: &Path, skill_id: &str) -> bool {
    let mut store = load_outcomes(path);
    let existed = store.rubrics.remove(skill_id).is_some();
    if existed {
        save_outcomes(path, &store);
    }
    existed
}

pub fn assess_skill(
    skill_id: &str,
    content_length: usize,
    code_blocks: usize,
    step_count: usize,
    tech_term_count: usize,
    store: &OutcomeStore,
) -> Option<OutcomeAssessment> {
    let rubric = store.rubrics.get(skill_id)?;
    let mut criterion_results = Vec::new();
    let mut total_score = 0.0;
    let mut criteria_count = 0;

    for criterion in &rubric.criteria {
        let actual = get_metric_value(
            &criterion.metric_field,
            content_length,
            code_blocks,
            step_count,
            tech_term_count,
        );

        let status = if actual >= criterion.good_threshold {
            CriterionStatus::Good
        } else if actual >= criterion.needs_improvement_threshold {
            CriterionStatus::NeedsImprovement
        } else {
            CriterionStatus::Poor
        };

        let criterion_score = match status {
            CriterionStatus::Good => 1.0,
            CriterionStatus::NeedsImprovement => 0.5,
            CriterionStatus::Poor => 0.0,
            CriterionStatus::NotApplicable => 0.0,
        };

        criterion_results.push(CriterionResult {
            name: criterion.name.clone(),
            actual_value: actual,
            threshold_good: criterion.good_threshold,
            threshold_needs_improvement: criterion.needs_improvement_threshold,
            status,
        });

        total_score += criterion_score;
        criteria_count += 1;
    }

    let avg_score = if criteria_count > 0 {
        (total_score / criteria_count as f64) * 100.0
    } else {
        0.0
    };

    let level = if avg_score >= 80.0 {
        "Good"
    } else if avg_score >= 50.0 {
        "Needs Improvement"
    } else {
        "Poor"
    };

    Some(OutcomeAssessment {
        skill_id: skill_id.to_string(),
        rubric_name: rubric.skill_id.clone(),
        level: level.to_string(),
        score: avg_score,
        criterion_results,
    })
}

fn get_metric_value(
    field: &str,
    content_length: usize,
    code_blocks: usize,
    step_count: usize,
    tech_term_count: usize,
) -> f64 {
    match field {
        "content_length" => content_length as f64,
        "code_blocks" => code_blocks as f64,
        "step_count" => step_count as f64,
        "tech_term_count" => tech_term_count as f64,
        "complexity_score" => compute_complexity(content_length, code_blocks, step_count, tech_term_count),
        _ => 0.0,
    }
}

fn compute_complexity(
    content_length: usize,
    code_blocks: usize,
    step_count: usize,
    tech_term_count: usize,
) -> f64 {
    let mut c = 0.0;
    c += (content_length as f64 / 100.0).min(30.0);
    c += (code_blocks as f64 * 5.0).min(25.0);
    c += (step_count as f64 * 2.0).min(20.0);
    c += (tech_term_count as f64 * 2.5).min(25.0);
    if code_blocks > 0 && step_count > 5 && tech_term_count > 3 {
        c += 10.0;
    }
    c
}

pub fn print_outcome_assessment(assessment: &OutcomeAssessment) {
    println!("  Outcome Assessment ({}):", assessment.rubric_name);
    println!("    Level: {} (Score: {:.0}/100)", assessment.level, assessment.score);
    println!("    Criteria:");
    for cr in &assessment.criterion_results {
        let icon = match cr.status {
            CriterionStatus::Good => "✓",
            CriterionStatus::NeedsImprovement => "~",
            CriterionStatus::Poor => "✗",
            CriterionStatus::NotApplicable => "-",
        };
        println!("      {} {}: {:.0} (good ≥ {}, needs ≥ {})", icon, cr.name, cr.actual_value, cr.threshold_good, cr.threshold_needs_improvement);
    }
}

pub fn print_rubric_list(store: &OutcomeStore) {
    if store.rubrics.is_empty() {
        println!("No outcome rubrics defined.");
        println!("Use: medusa outcome-add <skill_id> to add a rubric");
        return;
    }
    println!("\n=== Outcome Rubrics ===");
    for (id, rubric) in &store.rubrics {
        println!("  {} ({} criteria)", id, rubric.criteria.len());
        for criterion in &rubric.criteria {
            println!("    - {}: good ≥ {}, needs ≥ {} (field: {})", 
                criterion.name, criterion.good_threshold, criterion.needs_improvement_threshold, criterion.metric_field);
        }
    }
}

pub fn get_default_rubric(skill_id: &str) -> OutcomeRubric {
    OutcomeRubric {
        skill_id: skill_id.to_string(),
        criteria: vec![
            OutcomeCriterion {
                name: "Content Length".to_string(),
                description: "Skill content should be comprehensive".to_string(),
                good_threshold: 3000.0,
                needs_improvement_threshold: 1000.0,
                metric_field: "content_length".to_string(),
            },
            OutcomeCriterion {
                name: "Code Examples".to_string(),
                description: "Practical code examples demonstrate applied knowledge".to_string(),
                good_threshold: 5.0,
                needs_improvement_threshold: 2.0,
                metric_field: "code_blocks".to_string(),
            },
            OutcomeCriterion {
                name: "Step Instructions".to_string(),
                description: "Clear step-by-step guidance for implementation".to_string(),
                good_threshold: 10.0,
                needs_improvement_threshold: 5.0,
                metric_field: "step_count".to_string(),
            },
            OutcomeCriterion {
                name: "Technical Depth".to_string(),
                description: "Use of technical terminology indicates depth".to_string(),
                good_threshold: 15.0,
                needs_improvement_threshold: 5.0,
                metric_field: "tech_term_count".to_string(),
            },
        ],
        levels: {
            let mut m = HashMap::new();
            m.insert("Good".to_string(), "Skill meets quality standards".to_string());
            m.insert("Needs Improvement".to_string(), "Skill has some gaps but shows effort".to_string());
            m.insert("Poor".to_string(), "Skill needs significant improvement".to_string());
            m
        },
    }
}
