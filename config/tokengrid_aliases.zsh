# TokenGrid Automatic Agent Routing Aliases
# Add this line to your ~/.zshrc:
#   source /home/raven/Devspace/Cybercore/ai/omarchy-plugins/tokengrid/config/tokengrid_aliases.zsh

TOKENGRID_WRAPPER="$HOME/Devspace/Cybercore/ai/omarchy-plugins/tokengrid/bin/agent_wrapper.sh"

_tokengrid_run() {
    local agent_cmd="$1"
    shift
    if [ -x "$TOKENGRID_WRAPPER" ]; then
        "$TOKENGRID_WRAPPER" "$agent_cmd" "$@"
    else
        command "$agent_cmd" "$@"
    fi
}

# Define wrappers for all 17 target agents
gemini() { _tokengrid_run "gemini" "$@"; }
claude() { _tokengrid_run "claude" "$@"; }
crush() { _tokengrid_run "crush" "$@"; }
cursor-agent() { _tokengrid_run "cursor-agent" "$@"; }
codex() { _tokengrid_run "codex" "$@"; }
copilot() { _tokengrid_run "copilot" "$@"; }
oh-my-pi() { _tokengrid_run "oh-my-pi" "$@"; }
pi() { _tokengrid_run "pi" "$@"; }
opencode() { _tokengrid_run "opencode" "$@"; }
grok() { _tokengrid_run "grok" "$@"; }
ori() { _tokengrid_run "ori" "$@"; }
kimi() { _tokengrid_run "kimi" "$@"; }
muse() { _tokengrid_run "muse" "$@"; }
antigravity-cli() { _tokengrid_run "antigravity-cli" "$@"; }
qwen() { _tokengrid_run "qwen" "$@"; }
devin() { _tokengrid_run "devin" "$@"; }
hermes() { _tokengrid_run "hermes" "$@"; }
hermes-agent() { _tokengrid_run "hermes-agent" "$@"; }
kilocode() { _tokengrid_run "kilocode" "$@"; }
