use cv_writer_mcp::compiler::LatexCompiler;
use cv_writer_mcp::schema::sample_cv_profile;
use cv_writer_mcp::template::render_latex;

#[tokio::test]
async fn test_compile_sample_profile_to_pdf() {
    let compiler = LatexCompiler::new().expect("LuaLaTeX not installed on host");
    let profile = sample_cv_profile();
    let rendered_tex = render_latex(&profile).expect("Failed to render LaTeX");

    let res = compiler
        .compile(&rendered_tex)
        .await
        .expect("Compilation failed");

    assert!(!res.pdf_bytes.is_empty());
    // PDF Magic bytes: %PDF-
    assert_eq!(&res.pdf_bytes[0..5], b"%PDF-");
}

#[tokio::test]
async fn test_compile_with_special_latex_chars() {
    let compiler = LatexCompiler::new().expect("LuaLaTeX not installed on host");
    let mut profile = sample_cv_profile();

    profile.contact.name = "John & Jane Doe #1".to_string();
    profile.summary = Some(
        "Working on C++ & C#, 100% test coverage, $100k budget, path_with_underscore, {braces}, ~tilde, ^hat."
            .to_string(),
    );

    let rendered_tex = render_latex(&profile).expect("Failed to render LaTeX");
    assert!(rendered_tex.contains("John \\& Jane Doe \\#1"));
    assert!(rendered_tex.contains("100\\%"));

    let res = compiler
        .compile(&rendered_tex)
        .await
        .expect("Compilation failed with escaped chars");
    assert_eq!(&res.pdf_bytes[0..5], b"%PDF-");
}

#[tokio::test]
async fn test_compile_long_location_atomic_mbox() {
    let compiler = LatexCompiler::new().expect("LuaLaTeX not installed on host");
    let mut profile = sample_cv_profile();

    profile.contact.location =
        Some("Nha Trang, Vietnam (Relocating to Ho Chi Minh City)".to_string());

    let rendered_tex = render_latex(&profile).expect("Failed to render LaTeX");
    assert!(rendered_tex.contains(r"\mbox{{ \color{gray}\faMapMarker* }~Nha Trang, Vietnam (Relocating to Ho Chi Minh City)}"));

    let res = compiler
        .compile(&rendered_tex)
        .await
        .expect("Compilation failed for long atomic location");
    assert_eq!(&res.pdf_bytes[0..5], b"%PDF-");
}

#[tokio::test]
async fn test_compile_long_education_and_experience_no_overflow() {
    let compiler = LatexCompiler::new().expect("LuaLaTeX not installed on host");
    let mut profile = sample_cv_profile();

    profile.education = vec![cv_writer_mcp::schema::EducationItem {
        institution: "Ho Chi Minh City University of Technology and Education, Vietnam National University"
            .to_string(),
        degree: Some(
            "Bachelor of Engineering in Artificial Intelligence, Computer Vision and Robotics"
                .to_string(),
        ),
        dates: Some("Sep 2018 -- Jun 2023".to_string()),
        highlights: vec![
            "Graduated with High Distinction, Top 1% Valedictorian candidate.".to_string(),
        ],
    }];

    profile.experience = vec![cv_writer_mcp::schema::ExperienceItem {
        company: "Amazon Web Services Infrastructure & Cloud Scalability Engineering Group".to_string(),
        location: Some("Seattle, WA".to_string()),
        roles: vec![cv_writer_mcp::schema::RoleItem {
            title: "Lead Principal Cloud Infrastructure and Distributed Systems Reliability Architect".to_string(),
            dates: Some("Jul 2020 -- Mar 2022".to_string()),
            highlights: vec!["Increased throughput by 823%.".to_string()],
        }],
    }];

    let rendered_tex = render_latex(&profile).expect("Failed to render LaTeX");
    assert!(rendered_tex.contains(r"\cventry{Ho Chi Minh City University of Technology and Education"));
    assert!(rendered_tex.contains(r"\cvsubentry{Lead Principal Cloud Infrastructure"));

    let res = compiler
        .compile(&rendered_tex)
        .await
        .expect("Compilation failed for long wording profile");
    assert_eq!(&res.pdf_bytes[0..5], b"%PDF-");
    // Verify LuaLaTeX log contains no overfull \hbox warnings
    assert!(
        !res.log.to_lowercase().contains("overfull \\hbox"),
        "LuaLaTeX emitted Overfull \\hbox warnings indicating edge clipping:\n{}",
        res.log
    );
}
