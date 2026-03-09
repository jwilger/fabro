use tracing::{debug, info};

use arc_github::{self as github_app, ssh_url_to_https, GitHubAppCredentials};

/// Derive a PR title from the workflow goal.
///
/// Uses the first line, truncated to 120 characters for readability.
fn pr_title_from_goal(goal: &str) -> String {
    let first_line = goal.lines().next().unwrap_or(goal);
    if first_line.chars().count() > 120 {
        let truncated: String = first_line.chars().take(119).collect();
        format!("{truncated}…")
    } else {
        first_line.to_string()
    }
}

/// Truncate a PR body to fit GitHub's 65,536 character limit.
fn truncate_pr_body(body: &str) -> String {
    const MAX_BODY: usize = 65_536;
    const SUFFIX: &str = "\n\n_(truncated)_";
    if body.len() <= MAX_BODY {
        return body.to_string();
    }
    let cutoff = MAX_BODY - SUFFIX.len();
    format!("{}{SUFFIX}", &body[..cutoff])
}

/// Generate a PR body from the diff and goal using an LLM.
pub async fn generate_pr_body(diff: &str, goal: &str, model: &str) -> Result<String, String> {
    let system = "Write a concise PR description summarizing the changes.".to_string();

    // Truncate diff to fit context windows (~50k chars)
    let max_diff_len = 50_000;
    let truncated_diff = if diff.len() > max_diff_len {
        &diff[..max_diff_len]
    } else {
        diff
    };

    let prompt = format!("Goal: {goal}\n\nDiff:\n```\n{truncated_diff}\n```");

    let params = arc_llm::generate::GenerateParams::new(model)
        .system(system)
        .prompt(prompt);

    let result = arc_llm::generate::generate(params)
        .await
        .map_err(|e| format!("LLM generation failed: {e}"))?;

    Ok(result.response.text())
}

/// Optionally open a pull request after a successful workflow run.
///
/// Returns `Ok(Some(html_url))` if a PR was created, `Ok(None)` if the diff
/// was empty, or `Err` on failure.
pub async fn maybe_open_pull_request(
    creds: &GitHubAppCredentials,
    origin_url: &str,
    base_branch: &str,
    head_branch: &str,
    goal: &str,
    diff: &str,
    model: &str,
) -> Result<Option<String>, String> {
    if diff.is_empty() {
        debug!("Empty diff, skipping pull request creation");
        return Ok(None);
    }

    let https_url = ssh_url_to_https(origin_url);
    let (owner, repo) = github_app::parse_github_owner_repo(&https_url)?;

    let body = generate_pr_body(diff, goal, model).await?;
    let body = truncate_pr_body(&body);

    let title = pr_title_from_goal(goal);

    let (url, pr_number) = github_app::create_pull_request(
        creds,
        &owner,
        &repo,
        base_branch,
        head_branch,
        &title,
        &body,
    )
    .await?;

    info!(pr_url = %url, pr_number, "Pull request created");

    Ok(Some(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_title_uses_first_line() {
        let goal = "# Add Draft PR Mode\n\nMore details here...";
        assert_eq!(pr_title_from_goal(goal), "# Add Draft PR Mode");
    }

    #[test]
    fn pr_title_truncates_long_line() {
        let long = "x".repeat(300);
        let title = pr_title_from_goal(&long);
        assert_eq!(title.chars().count(), 120);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn pr_body_truncates_long_body() {
        let long = "x".repeat(70_000);
        let body = truncate_pr_body(&long);
        assert!(body.len() <= 65_536);
        assert!(body.ends_with("\n\n_(truncated)_"));
    }

    #[test]
    fn pr_body_short_body_unchanged() {
        let short = "Some PR description";
        assert_eq!(truncate_pr_body(short), short);
    }

    #[test]
    fn pr_title_short_goal_unchanged() {
        assert_eq!(pr_title_from_goal("Fix bug"), "Fix bug");
    }

    #[tokio::test]
    async fn empty_diff_returns_none() {
        let creds = GitHubAppCredentials {
            app_id: "123".to_string(),
            private_key_pem: "unused".to_string(),
        };
        let result = maybe_open_pull_request(
            &creds,
            "https://github.com/owner/repo.git",
            "main",
            "arc/run/123",
            "Fix bug",
            "",
            "claude-sonnet-4-20250514",
        )
        .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
