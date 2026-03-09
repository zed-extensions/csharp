use zed_extension_api::{self as zed, settings::LspSettings};

/// Try to find the solution file for the worktree.
/// Priority: user setting > auto-detect by folder name.
pub fn resolve_solution(worktree: &zed::Worktree) -> Result<Option<String>, String> {
    if let Some(configured) = user_configured_solution(worktree) {
        if worktree.read_text_file(&configured).is_ok() {
            return Ok(Some(configured));
        }
        return Err(format!(
            "Configured solution '{}' not found. Check your lsp.roslyn.settings.solution setting.",
            configured
        ));
    }

    Ok(auto_detect_solution(worktree))
}

fn user_configured_solution(worktree: &zed::Worktree) -> Option<String> {
    let settings = LspSettings::for_worktree("roslyn", worktree).ok()?;
    let settings_value = settings.settings?;
    settings_value
        .get("solution")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn auto_detect_solution(worktree: &zed::Worktree) -> Option<String> {
    let root = worktree.root_path();
    let project_name = root
        .rsplit('/')
        .find(|s| !s.is_empty())
        .or_else(|| root.rsplit('\\').find(|s| !s.is_empty()))
        .unwrap_or(&root);

    let primary = format!("{project_name}.sln");
    if worktree.read_text_file(&primary).is_ok() {
        return Some(primary);
    }

    None
}
