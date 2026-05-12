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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> OutcomeStore {
        let mut store = OutcomeStore::default();
        store.rubrics.insert("test-skill".to_string(), OutcomeRubric {
            skill_id: "test-skill".to_string(),
            criteria: vec![
                OutcomeCriterion {
                    name: "Content".to_string(),
                    description: "Content length".to_string(),
                    good_threshold: 1000.0,
                    needs_improvement_threshold: 500.0,
                    metric_field: "content_length".to_string(),
                },
                OutcomeCriterion {
                    name: "Code".to_string(),
                    description: "Code blocks".to_string(),
                    good_threshold: 5.0,
                    needs_improvement_threshold: 2.0,
                    metric_field: "code_blocks".to_string(),
                },
            ],
            levels: HashMap::new(),
        });
        store
    }

    #[test]
    fn test_assess_skill_good() {
        let store = test_store();
        let assessment = assess_skill("test-skill", 2000, 10, 15, 20, &store);
        assert!(assessment.is_some());
        let a = assessment.unwrap();
        assert_eq!(a.skill_id, "test-skill");
        assert_eq!(a.criterion_results.len(), 2);
        // Both should be Good
        assert_eq!(a.criterion_results[0].status, CriterionStatus::Good);
        assert_eq!(a.criterion_results[1].status, CriterionStatus::Good);
        assert!((a.score - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_assess_skill_needs_improvement() {
        let store = test_store();
        let assessment = assess_skill("test-skill", 700, 3, 5, 8, &store);
        assert!(assessment.is_some());
        let a = assessment.unwrap();
        // Content: 700 < 1000 good but >= 500 needs_improvement → NeedsImprovement (0.5)
        // Code: 3 >= 2 needs_improvement but < 5 good → NeedsImprovement (0.5)
        assert_eq!(a.criterion_results[0].status, CriterionStatus::NeedsImprovement);
        assert_eq!(a.criterion_results[1].status, CriterionStatus::NeedsImprovement);
        assert!((a.score - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_assess_skill_poor() {
        let store = test_store();
        let assessment = assess_skill("test-skill", 100, 0, 1, 0, &store);
        assert!(assessment.is_some());
        let a = assessment.unwrap();
        assert_eq!(a.criterion_results[0].status, CriterionStatus::Poor);
        assert_eq!(a.criterion_results[1].status, CriterionStatus::Poor);
        assert!((a.score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_assess_skill_no_rubric() {
        let store = OutcomeStore::default();
        let assessment = assess_skill("unknown-skill", 1000, 5, 10, 10, &store);
        assert!(assessment.is_none());
    }

    #[test]
    fn test_assess_skill_mixed_results() {
        let store = test_store();
        let assessment = assess_skill("test-skill", 1500, 1, 8, 12, &store);
        assert!(assessment.is_some());
        let a = assessment.unwrap();
        // Content: 1500 >= 1000 → Good (1.0)
        // Code: 1 < 2 → Poor (0.0)
        assert_eq!(a.criterion_results[0].status, CriterionStatus::Good);
        assert_eq!(a.criterion_results[1].status, CriterionStatus::Poor);
        // Average: (1.0 + 0.0) / 2 = 0.5 → 50.0
        assert!((a.score - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_add_and_remove_rubric() {
        let temp_dir = std::env::temp_dir().join("medusa_outcomes_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let rubric = OutcomeRubric {
            skill_id: "test-skill".to_string(),
            criteria: vec![OutcomeCriterion {
                name: "Test".to_string(),
                description: "Test criterion".to_string(),
                good_threshold: 10.0,
                needs_improvement_threshold: 5.0,
                metric_field: "content_length".to_string(),
            }],
            levels: HashMap::new(),
        };

        add_rubric(&temp_dir, rubric);
        let store = load_outcomes(&temp_dir);
        assert!(store.rubrics.contains_key("test-skill"));

        let removed = remove_rubric(&temp_dir, "test-skill");
        assert!(removed);
        let store = load_outcomes(&temp_dir);
        assert!(!store.rubrics.contains_key("test-skill"));

        let removed_again = remove_rubric(&temp_dir, "nonexistent");
        assert!(!removed_again);

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_get_default_rubric() {
        let rubric = get_default_rubric("my-skill");
        assert_eq!(rubric.skill_id, "my-skill");
        assert_eq!(rubric.criteria.len(), 4);
        // Check metric fields
        let fields: Vec<&str> = rubric.criteria.iter().map(|c| c.metric_field.as_str()).collect();
        assert!(fields.contains(&"content_length"));
        assert!(fields.contains(&"code_blocks"));
        assert!(fields.contains(&"step_count"));
        assert!(fields.contains(&"tech_term_count"));
    }

    #[test]
    fn test_get_metric_value() {
        assert_eq!(get_metric_value("content_length", 5000, 5, 10, 20), 5000.0);
        assert_eq!(get_metric_value("code_blocks", 5000, 5, 10, 20), 5.0);
        assert_eq!(get_metric_value("step_count", 5000, 5, 10, 20), 10.0);
        assert_eq!(get_metric_value("tech_term_count", 5000, 5, 10, 20), 20.0);
        assert_eq!(get_metric_value("complexity_score", 3000, 8, 12, 18), 110.0);
        assert_eq!(get_metric_value("unknown_field", 1000, 5, 10, 20), 0.0);
    }

    #[test]
    fn test_compute_complexity() {
        // All metrics at zero
        assert_eq!(compute_complexity(0, 0, 0, 0), 0.0);
        // With bonus (code_blocks > 0, step_count > 5, tech_term_count > 3)
        // 30 + 25 + 20 + 25 + 10 = 110
        let c = compute_complexity(5000, 6, 10, 10);
        assert_eq!(c, 110.0);
        // Without bonus
        // 30 + 10 + 6 + 5 = 51
        let c = compute_complexity(5000, 2, 3, 2);
        assert_eq!(c, 51.0);
    }
}
