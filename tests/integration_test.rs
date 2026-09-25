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
