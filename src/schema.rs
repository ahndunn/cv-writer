use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Canonical CV / Resume Profile Model matching the Star Rover template structure.
/// Designed for AI agents to synthesize arbitrary user resumes into an abstracted, typed interface.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CvProfile {
    /// Personal contact information and social links.
    pub contact: ContactInfo,
    /// Optional professional summary or executive objective.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Education entries (university, degree, dates, coursework/honors).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub education: Vec<EducationItem>,
    /// Professional work experience grouped by company and sub-roles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub experience: Vec<ExperienceItem>,
    /// Notable engineering, research, or personal projects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectItem>,
    /// Skills classified by category (e.g., Technical, Languages, Tools).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<SkillCategory>,
    /// Professional certifications, licenses, or bootcamps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub certifications: Vec<CertificationItem>,
    /// Academic publications, research papers, or articles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub publications: Vec<PublicationItem>,
    /// Honors, awards, or recognitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub awards: Vec<AwardItem>,
    /// Option to hide footer page numbering (default: false).
    #[serde(default)]
    pub hide_page_numbers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ContactInfo {
    /// Full name of the candidate (e.g. "Jane Doe").
    pub name: String,
    /// Phone number (e.g. "+1 (555) 019-2834").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Email address (e.g. "jane.doe@example.com").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// GitHub username or profile handle (e.g. "janedoe").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    /// LinkedIn profile handle (e.g. "jane-doe").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,
    /// Personal portfolio, blog, or website URL (e.g. "https://janedoe.dev").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Geographic location or city, state/country (e.g. "San Francisco, CA").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EducationItem {
    /// University, college, or educational institution name.
    pub institution: String,
    /// Degree or major (e.g. "B.S. in Computer Science & Engineering").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degree: Option<String>,
    /// Dates attended or graduation year (e.g. "2020 -- 2024").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<String>,
    /// Bullet points for honors, GPA, scholarships, or relevant coursework.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ExperienceItem {
    /// Organization or company name (e.g. "Google").
    pub company: String,
    /// Location of the office (e.g. "Mountain View, CA").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Positions/roles held at this company (supports multiple hierarchical roles).
    pub roles: Vec<RoleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RoleItem {
    /// Job title / designation (e.g. "Senior Distributed Systems Engineer").
    pub title: String,
    /// Duration / date range (e.g. "Jul 2021 -- Present").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<String>,
    /// Bullet points of accomplishments, metrics, and responsibilities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ProjectItem {
    /// Project name.
    pub name: String,
    /// Optional web or repository link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Date or duration of the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dates: Option<String>,
    /// Project details, technologies utilized, and measurable outcomes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SkillCategory {
    /// Category name (e.g. "Languages", "Cloud & Infra", "Frameworks").
    pub category: String,
    /// List of skill names under this category.
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CertificationItem {
    /// Name of certification or bootcamp.
    pub name: String,
    /// Issuing body or institution (e.g. "AWS", "Linux Foundation").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Year or date received.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PublicationItem {
    /// Full citation text for publication or paper.
    pub citation: String,
    /// Optional link to paper (arXiv, DOI, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct AwardItem {
    /// Title of the award or honor.
    pub title: String,
    /// Year or date received.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Short summary or context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Helper function to provide a sample CV matching Star Rover demonstration content.
pub fn sample_cv_profile() -> CvProfile {
    CvProfile {
        contact: ContactInfo {
            name: "Star Rover".to_string(),
            phone: Some("+1 (123) 456-7890".to_string()),
            email: Some("email@example.com".to_string()),
            github: Some("yourusername".to_string()),
            linkedin: Some("profile-name".to_string()),
            website: Some("https://starportfolio.com".to_string()),
            location: Some("San Francisco, CA".to_string()),
        },
        summary: Some("Systems and backend engineer specialized in high-performance stateless distributed microservices, Rust, and modern typesetting architectures.".to_string()),
        education: vec![
            EducationItem {
                institution: "Graduation University".to_string(),
                degree: Some("BS in Computer Science & Engineering".to_string()),
                dates: Some("2020 -- 2024".to_string()),
                highlights: vec![
                    "summa cum laude".to_string(),
                    "Extensive coursework in Distributed Systems, Operating Systems, Statistics".to_string(),
                    "Van Damme Scholarship recipient".to_string(),
                ],
            },
        ],
        experience: vec![
            ExperienceItem {
                company: "Amazon".to_string(),
                location: Some("Seattle, WA".to_string()),
                roles: vec![
                    RoleItem {
                        title: "Prime Infrastructure Engineer".to_string(),
                        dates: Some("Jul 2020 -- Mar 2022".to_string()),
                        highlights: vec![
                            "Increased order volume throughput by 823% over continuous deployment iterations.".to_string(),
                            "Researched diverse product catalog caching optimizations.".to_string(),
                            "Regularly contributed to platform reliability and performance portals.".to_string(),
                        ],
                    },
                ],
            },
            ExperienceItem {
                company: "Google".to_string(),
                location: Some("Mountain View, CA".to_string()),
                roles: vec![
                    RoleItem {
                        title: "Backend Engineer".to_string(),
                        dates: Some("Jan 2014 -- Dec 2016".to_string()),
                        highlights: vec![
                            "Responsible for managing indexing pipelines across search and distributed storage suites.".to_string(),
                            "Troubleshot high-concurrency calendar sync bottlenecks.".to_string(),
                        ],
                    },
                    RoleItem {
                        title: "Software Engineering Intern".to_string(),
                        dates: Some("Jun 2013 -- Dec 2013".to_string()),
                        highlights: vec![
                            "Ensured timely execution and telemetry observability of cluster automation tasks.".to_string(),
                            "Exceeded market benchmarks for throughput latency SLAs.".to_string(),
                        ],
                    },
                ],
            },
        ],
        projects: vec![
            ProjectItem {
                name: "Lunar Rover Automation".to_string(),
                url: Some("https://robotics.nasa.gov/lmr/".to_string()),
                dates: Some("2012 -- 2013".to_string()),
                highlights: vec![
                    "LIDAR array telemetry, GPS synchronization, Variable FOV camera pipeline in Rust and C++.".to_string(),
                    "Designed system to be reconfigurable and modular for arbitrary sensor payloads.".to_string(),
                ],
            },
            ProjectItem {
                name: "Mars Autonomous Sensor Pod".to_string(),
                url: Some("https://science.nasa.gov/mission/mars-exploration-rovers/".to_string()),
                dates: Some("Feb 2012".to_string()),
                highlights: vec![
                    "Implemented real-time sensor processing and fault recovery algorithms.".to_string(),
                    "Optimized battery duty cycles by 50% under simulated cold martian nights.".to_string(),
                ],
            },
        ],
        skills: vec![
            SkillCategory {
                category: "Technical".to_string(),
                items: vec!["Rust".to_string(), "C/C++".to_string(), "Linux/POSIX".to_string(), "Docker".to_string(), "LuaLaTeX".to_string()],
            },
            SkillCategory {
                category: "Languages".to_string(),
                items: vec!["English (Native)".to_string(), "Vietnamese (Fluent)".to_string()],
            },
            SkillCategory {
                category: "Platforms".to_string(),
                items: vec!["Kubernetes".to_string(), "AWS".to_string(), "GCP".to_string()],
            },
        ],
        certifications: vec![
            CertificationItem {
                name: "Certified Kubernetes Administrator (CKA)".to_string(),
                issuer: Some("Linux Foundation".to_string()),
                date: Some("2023".to_string()),
            },
            CertificationItem {
                name: "AWS Certified Solutions Architect -- Professional".to_string(),
                issuer: Some("Amazon Web Services".to_string()),
                date: Some("2022".to_string()),
            },
        ],
        publications: vec![
            PublicationItem {
                citation: "A. Vaswani et al., \"Attention is All you Need,\" arXiv (Cornell University), vol. 30, pp. 5998-6008, Jun. 2017.".to_string(),
                url: Some("https://arxiv.org/pdf/1706.03762v5".to_string()),
            },
        ],
        awards: vec![
            AwardItem {
                title: "ACM Collegiate Programming Finalist".to_string(),
                date: Some("2023".to_string()),
                summary: Some("Ranked top 1% internationally in algorithmic efficiency.".to_string()),
            },
        ],
        hide_page_numbers: false,
    }
}
