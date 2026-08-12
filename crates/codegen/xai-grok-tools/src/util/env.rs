//! Environment variable helpers and process isolation for terminal execution.
//!
//! TTY detachment and pager helpers live in the lightweight
//! [`xai_tty_utils`] crate and are re-exported here. Agent markers and sampler
//! credential scrubbing stay in this crate because they are Grok-tool policy,
//! not general terminal utilities.

pub use xai_tty_utils::{detach_from_tty, pager_env};

/// The positive-integer env contract shared by limits and timeouts: plain
/// digits only; `None` for anything else, including zero.
pub fn parse_positive(value: &str) -> Option<u64> {
    value.parse::<u64>().ok().filter(|&parsed| parsed > 0)
}

/// Parse a positive whole-number env value into a count. A set-but-invalid
/// value warns and reads as unset, so the caller's default applies.
pub fn parse_positive_env(var: &str, value: Option<String>) -> Option<usize> {
    let value = value?;
    let parsed = parse_positive(&value).and_then(|parsed| usize::try_from(parsed).ok());
    if parsed.is_none() {
        tracing::warn!(
            var,
            %value,
            "env value is not a positive whole number in plain digits; using the default"
        );
    }
    parsed
}

/// Env var set on agent-spawned terminal processes so host tools (e.g. `x ban`)
/// can distinguish agent invocations from human interactive shells.
/// Note: the CLI also uses `GROK_AGENT` as an
/// optional agent-definition selector for launching `grok` itself; child terminal
/// processes only need the sentinel value `"1"`.
pub const GROK_AGENT_ENV: &str = "GROK_AGENT";

/// Sentinel value for [`GROK_AGENT_ENV`] on agent tool terminals.
pub const GROK_AGENT_ENV_VALUE: &str = "1";

/// The OpenAI credential belongs to the sampler process and must never be
/// inherited by agent-controlled terminal commands.
pub const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";

/// Force `GROK_AGENT=1` on an agent terminal child so request/login env cannot
/// clear the agent marker.
pub fn apply_grok_agent_marker(cmd: &mut tokio::process::Command) {
    cmd.env(GROK_AGENT_ENV, GROK_AGENT_ENV_VALUE);
}

/// Remove the OpenAI credential from an agent-spawned child process.
///
/// Call this after every inherited, login-shell, policy, and per-request
/// environment layer so none of those layers can re-introduce the key.
pub fn scrub_openai_api_key(cmd: &mut tokio::process::Command) {
    cmd.env_remove(OPENAI_API_KEY_ENV);
}

/// Synchronous-command counterpart to [`scrub_openai_api_key`].
pub fn scrub_openai_api_key_std(cmd: &mut std::process::Command) {
    cmd.env_remove(OPENAI_API_KEY_ENV);
}

/// Expand the four plugin-path tokens (`${CLAUDE_PLUGIN_ROOT}` / `${GROK_PLUGIN_ROOT}`
/// and `${CLAUDE_PLUGIN_DATA}` / `${GROK_PLUGIN_DATA}`) in `s`. Each pair is expanded
/// only when its value is provided. Single source of truth for plugin agent bodies,
/// plugin skill/command bodies, and plugin MCP/hook config substitution.
pub fn substitute_plugin_tokens(
    s: &str,
    plugin_root: Option<&str>,
    plugin_data: Option<&str>,
) -> String {
    let mut out = s.to_string();
    if let Some(root) = plugin_root {
        out = out
            .replace("${CLAUDE_PLUGIN_ROOT}", root)
            .replace("${GROK_PLUGIN_ROOT}", root);
    }
    if let Some(data) = plugin_data {
        out = out
            .replace("${CLAUDE_PLUGIN_DATA}", data)
            .replace("${GROK_PLUGIN_DATA}", data);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        GROK_AGENT_ENV, GROK_AGENT_ENV_VALUE, OPENAI_API_KEY_ENV, scrub_openai_api_key,
        scrub_openai_api_key_std, substitute_plugin_tokens,
    };

    const ALL_TOKENS: &str = "${CLAUDE_PLUGIN_ROOT}/a ${GROK_PLUGIN_ROOT}/b ${CLAUDE_PLUGIN_DATA}/c ${GROK_PLUGIN_DATA}/d";

    #[test]
    fn expands_all_four_tokens_when_both_provided() {
        let out = substitute_plugin_tokens(ALL_TOKENS, Some("/root"), Some("/data"));
        assert_eq!(out, "/root/a /root/b /data/c /data/d");
    }

    #[test]
    fn leaves_tokens_literal_when_both_none() {
        let out = substitute_plugin_tokens(ALL_TOKENS, None, None);
        assert_eq!(out, ALL_TOKENS);
    }

    #[test]
    fn expands_only_root_when_data_none() {
        let out = substitute_plugin_tokens(ALL_TOKENS, Some("/root"), None);
        assert_eq!(
            out,
            "/root/a /root/b ${CLAUDE_PLUGIN_DATA}/c ${GROK_PLUGIN_DATA}/d"
        );
    }

    #[test]
    fn agent_marker_constants_match_cursor_parity() {
        assert_eq!(GROK_AGENT_ENV, "GROK_AGENT");
        assert_eq!(GROK_AGENT_ENV_VALUE, "1");
    }

    #[test]
    fn openai_key_scrub_preserves_benign_environment_entries() {
        let mut cmd = tokio::process::Command::new("true");
        cmd.env(OPENAI_API_KEY_ENV, "test-secret")
            .env("PATH", "/safe/bin");

        scrub_openai_api_key(&mut cmd);

        let scrubbed = cmd
            .as_std()
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new(OPENAI_API_KEY_ENV))
            .map(|(_, value)| value);
        let path = cmd
            .as_std()
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("PATH"))
            .and_then(|(_, value)| value);

        assert_eq!(scrubbed, Some(None));
        assert_eq!(path, Some(std::ffi::OsStr::new("/safe/bin")));
    }

    #[test]
    fn std_command_openai_key_scrub_allows_explicit_opt_in_afterward() {
        let mut cmd = std::process::Command::new("true");
        cmd.env(OPENAI_API_KEY_ENV, "inherited-secret")
            .env("PATH", "/safe/bin");

        scrub_openai_api_key_std(&mut cmd);
        // A trusted helper configuration may explicitly opt in after the
        // inherited credential has been removed.
        cmd.env(OPENAI_API_KEY_ENV, "explicit-secret");

        let envs: std::collections::HashMap<_, _> = cmd
            .get_envs()
            .filter_map(|(key, value)| Some((key.to_str()?, value?.to_str()?)))
            .collect();
        assert_eq!(envs.get(OPENAI_API_KEY_ENV), Some(&"explicit-secret"));
        assert_eq!(envs.get("PATH"), Some(&"/safe/bin"));
    }
}
