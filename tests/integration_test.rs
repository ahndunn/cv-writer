use cv_writer::cli::{Cli, Commands, CompileArgs};
use cv_writer::compiler::LatexCompiler;
use cv_writer::schema::sample_cv_profile;
use cv_writer::template::render_latex;
use tempfile::tempdir;

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
    assert!(rendered_tex.contains(
        r"\mbox{{ \color{gray}\faMapMarker* }~Nha Trang, Vietnam (Relocating to Ho Chi Minh City)}"
    ));

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

    profile.education = vec![cv_writer::schema::EducationItem {
        institution:
            "Ho Chi Minh City University of Technology and Education, Vietnam National University"
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

    profile.experience = vec![cv_writer::schema::ExperienceItem {
        company: "Amazon Web Services Infrastructure & Cloud Scalability Engineering Group"
            .to_string(),
        location: Some("Seattle, WA".to_string()),
        roles: vec![cv_writer::schema::RoleItem {
            title:
                "Lead Principal Cloud Infrastructure and Distributed Systems Reliability Architect"
                    .to_string(),
            dates: Some("Jul 2020 -- Mar 2022".to_string()),
            highlights: vec!["Increased throughput by 823%.".to_string()],
        }],
    }];

    let rendered_tex = render_latex(&profile).expect("Failed to render LaTeX");
    assert!(
        rendered_tex.contains(r"\cventry{Ho Chi Minh City University of Technology and Education")
    );
    assert!(rendered_tex.contains(r"\cvsubentry{Lead Principal Cloud Infrastructure"));

    let res = compiler
        .compile(&rendered_tex)
        .await
        .expect("Compilation failed for long wording profile");
    assert_eq!(&res.pdf_bytes[0..5], b"%PDF-");
    assert!(
        !res.log.to_lowercase().contains("overfull \\hbox"),
        "LuaLaTeX emitted Overfull \\hbox warnings indicating edge clipping:\n{}",
        res.log
    );
}

#[tokio::test]
async fn test_cli_compile_subcommand_end_to_end() {
    let dir = tempdir().unwrap();
    let profile_path = dir.path().join("profile.json");
    let output_pdf = dir.path().join("cv.pdf");
    let changelog_dir = dir.path().join("changelogs");

    let sample = sample_cv_profile();
    let json_bytes = serde_json::to_vec_pretty(&sample).unwrap();
    tokio::fs::write(&profile_path, json_bytes).await.unwrap();

    let cli = Cli {
        command: Commands::Compile(CompileArgs {
            profile: profile_path.clone(),
            output: output_pdf.clone(),
            changelog_dir: Some(changelog_dir.clone()),
            emit_tex: true,
            dry_run: false,
        }),
    };

    cli.execute().await.expect("CLI execution failed");

    // Verify PDF produced
    assert!(output_pdf.exists());
    let pdf_bytes = tokio::fs::read(&output_pdf).await.unwrap();
    assert_eq!(&pdf_bytes[0..5], b"%PDF-");

    // Verify TeX emitted
    let tex_path = dir.path().join("cv.tex");
    assert!(tex_path.exists());

    // Verify changelog directory created and contains markdown changelog
    assert!(changelog_dir.exists());
    let mut entries = tokio::fs::read_dir(&changelog_dir).await.unwrap();
    let mut found_md = false;
    let mut found_json = false;
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with("_changelog.md") {
            found_md = true;
        }
        if name.ends_with("_profile.json") {
            found_json = true;
        }
    }
    assert!(found_md, "Changelog markdown file was not generated");
    assert!(found_json, "Profile snapshot json file was not generated");
}
