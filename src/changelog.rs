use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::schema::CvProfile;

#[derive(Error, Debug)]
pub enum ChangelogError {
    #[error("I/O error during changelog processing: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Structured summary of changes between two profile states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileDiffReport {
    pub candidate_name: String,
    pub previous_timestamp: Option<String>,
    pub current_timestamp: String,
    pub summary_changed: bool,
    pub contact_changed: bool,
    pub education_changes: Vec<String>,
    pub experience_changes: Vec<String>,
    pub project_changes: Vec<String>,
    pub skill_changes: Vec<String>,
    pub cert_changes: Vec<String>,
    pub publication_changes: Vec<String>,
    pub award_changes: Vec<String>,
}

impl ProfileDiffReport {
    /// Format the diff report as clean, human- and LLM-friendly Markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "# CV Profile Changelog: {}\n\n",
            self.candidate_name
        ));
        md.push_str(&format!("- **Generated At**: {}\n", self.current_timestamp));
        if let Some(prev) = &self.previous_timestamp {
            md.push_str(&format!(
                "- **Compared Against Previous Baseline**: {}\n",
                prev
            ));
        } else {
            md.push_str("- **Baseline**: Initial profile version recorded.\n");
        }
        md.push_str("\n---\n\n");

        md.push_str("### Summary of Modifications\n\n");
        md.push_str(&format!(
            "- **Contact Information Modified**: {}\n",
            if self.contact_changed { "Yes" } else { "No" }
        ));
        md.push_str(&format!(
            "- **Executive Summary Modified**: {}\n",
            if self.summary_changed { "Yes" } else { "No" }
        ));

        append_section_diff(&mut md, "Experience", &self.experience_changes);
        append_section_diff(&mut md, "Projects", &self.project_changes);
        append_section_diff(&mut md, "Skills", &self.skill_changes);
        append_section_diff(&mut md, "Education", &self.education_changes);
        append_section_diff(&mut md, "Certifications", &self.cert_changes);
        append_section_diff(&mut md, "Publications", &self.publication_changes);
        append_section_diff(&mut md, "Awards", &self.award_changes);

        md
    }
}

fn append_section_diff(md: &mut String, title: &str, changes: &[String]) {
    md.push_str(&format!("\n### {}\n\n", title));
    if changes.is_empty() {
        md.push_str("- No changes detected.\n");
    } else {
        for change in changes {
            md.push_str(&format!("- {}\n", change));
        }
    }
}

/// Compute differences between an optional old profile and the current profile.
pub fn compute_profile_diff(
    old_profile: Option<&CvProfile>,
    new_profile: &CvProfile,
    prev_timestamp: Option<String>,
) -> ProfileDiffReport {
    let now = Utc::now().to_rfc3339();

    let Some(old) = old_profile else {
        return ProfileDiffReport {
            candidate_name: new_profile.contact.name.clone(),
            previous_timestamp: None,
            current_timestamp: now,
            summary_changed: false,
            contact_changed: false,
            education_changes: vec!["Initial education history registered.".to_string()],
            experience_changes: vec!["Initial work experience registered.".to_string()],
            project_changes: vec!["Initial project portfolio registered.".to_string()],
            skill_changes: vec!["Initial skill categories registered.".to_string()],
            cert_changes: vec![],
            publication_changes: vec![],
            award_changes: vec![],
        };
    };

    let summary_changed = old.summary != new_profile.summary;
    let contact_changed = old.contact != new_profile.contact;

    // Compare skills
    let mut skill_changes = Vec::new();
    let old_skills_flat: Vec<String> = old.skills.iter().flat_map(|sc| sc.items.clone()).collect();
    let new_skills_flat: Vec<String> = new_profile
        .skills
        .iter()
        .flat_map(|sc| sc.items.clone())
        .collect();

    for s in &new_skills_flat {
        if !old_skills_flat.contains(s) {
            skill_changes.push(format!("Added skill: `{}`", s));
        }
    }
    for s in &old_skills_flat {
        if !new_skills_flat.contains(s) {
            skill_changes.push(format!("Removed skill: `{}`", s));
        }
    }

    // Compare experience companies and roles
    let mut experience_changes = Vec::new();
    let old_companies: Vec<&String> = old.experience.iter().map(|e| &e.company).collect();
    let new_companies: Vec<&String> = new_profile.experience.iter().map(|e| &e.company).collect();

    for c in &new_companies {
        if !old_companies.contains(c) {
            experience_changes.push(format!("Added company experience: `{}`", c));
        }
    }
    for c in &old_companies {
        if !new_companies.contains(c) {
            experience_changes.push(format!("Removed company experience: `{}`", c));
        }
    }
    if old.experience != new_profile.experience && experience_changes.is_empty() {
        experience_changes
            .push("Updated roles, highlights, or dates within existing experiences.".to_string());
    }

    // Compare projects
    let mut project_changes = Vec::new();
    let old_projects: Vec<&String> = old.projects.iter().map(|p| &p.name).collect();
    let new_projects: Vec<&String> = new_profile.projects.iter().map(|p| &p.name).collect();

    for p in &new_projects {
        if !old_projects.contains(p) {
            project_changes.push(format!("Added project: `{}`", p));
        }
    }
    for p in &old_projects {
        if !new_projects.contains(p) {
            project_changes.push(format!("Removed project: `{}`", p));
        }
    }
    if old.projects != new_profile.projects && project_changes.is_empty() {
        project_changes.push("Updated project highlights, URLs, or dates.".to_string());
    }

    // Compare education
    let mut education_changes = Vec::new();
    if old.education != new_profile.education {
        education_changes.push(
            "Updated educational degrees, institutions, or coursework highlights.".to_string(),
        );
    }

    // Compare certs, publications, awards
    let mut cert_changes = Vec::new();
    if old.certifications != new_profile.certifications {
        cert_changes.push("Updated certification credentials.".to_string());
    }

    let mut publication_changes = Vec::new();
    if old.publications != new_profile.publications {
        publication_changes.push("Updated academic publications.".to_string());
    }

    let mut award_changes = Vec::new();
    if old.awards != new_profile.awards {
        award_changes.push("Updated honors or awards.".to_string());
    }

    ProfileDiffReport {
        candidate_name: new_profile.contact.name.clone(),
        previous_timestamp: prev_timestamp,
        current_timestamp: now,
        summary_changed,
        contact_changed,
        education_changes,
        experience_changes,
        project_changes,
        skill_changes,
        cert_changes,
        publication_changes,
        award_changes,
    }
}

/// Find the latest profile snapshot stored in the changelog directory, if any.
pub fn find_latest_profile(changelog_dir: &Path) -> Option<(CvProfile, String, PathBuf)> {
    if !changelog_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(changelog_dir).ok()?;
    let mut profile_files: Vec<PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let is_profile_json = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.ends_with("_profile.json"));
            if is_profile_json {
                profile_files.push(path);
            }
        }
    }

    // Sort ascending by file name (timestamps format YYYY-MM-DDTHH-MM-SSZ guarantees chronological order)
    profile_files.sort();

    let latest_path = profile_files.pop()?;
    let content = fs::read_to_string(&latest_path).ok()?;
    let profile: CvProfile = serde_json::from_str(&content).ok()?;

    let timestamp_str = latest_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_suffix("_profile"))
        .unwrap_or("unknown")
        .replace('-', ":");

    Some((profile, timestamp_str, latest_path))
}

/// Record a snapshot and changelog markdown report into the changelog directory.
pub fn record_changelog(
    changelog_dir: &Path,
    new_profile: &CvProfile,
) -> Result<ProfileDiffReport, ChangelogError> {
    fs::create_dir_all(changelog_dir)?;

    let (latest_profile, prev_ts) = match find_latest_profile(changelog_dir) {
        Some((p, ts, _)) => (Some(p), Some(ts)),
        None => (None, None),
    };

    let diff = compute_profile_diff(latest_profile.as_ref(), new_profile, prev_ts);
    let md = diff.to_markdown();

    // Format filename friendly timestamp (e.g. 2026-09-26T23-45-00Z)
    let file_ts = Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
    let snapshot_file = changelog_dir.join(format!("{}_profile.json", file_ts));
    let changelog_file = changelog_dir.join(format!("{}_changelog.md", file_ts));

    // Save JSON snapshot
    let json_bytes = serde_json::to_vec_pretty(new_profile)?;
    fs::write(snapshot_file, json_bytes)?;

    // Save Markdown changelog
    fs::write(changelog_file, md)?;

    Ok(diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::sample_cv_profile;
    use tempfile::tempdir;

    #[test]
    fn test_record_changelog_baseline_and_update() {
        let dir = tempdir().unwrap();
        let p1 = sample_cv_profile();

        // 1. Initial record
        let diff1 = record_changelog(dir.path(), &p1).unwrap();
        assert!(diff1.previous_timestamp.is_none());
        assert!(!diff1.experience_changes.is_empty());

        // 2. Updated record with a new skill and altered summary
        let mut p2 = p1.clone();
        p2.summary = Some("Updated executive summary.".to_string());
        p2.skills[0].items.push("Golang".to_string());

        let diff2 = record_changelog(dir.path(), &p2).unwrap();
        assert!(diff2.summary_changed);
        assert!(diff2.skill_changes.iter().any(|s| s.contains("Golang")));
    }
}
