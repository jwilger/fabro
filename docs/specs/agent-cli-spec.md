# Agent CLI Specification

This document specifies a command-line interface for the coding agent library defined in the [Coding Agent Loop Specification](./coding-agent-loop-spec.md). It is a thin wrapper that configures and runs a `Session`, designed to be implementable from scratch by any developer or coding agent in any programming language.

---

## Table of Contents

1. [Overview and Goals](#1-overview-and-goals)
2. [Invocation](#2-invocation)
3. [Permission Model](#3-permission-model)
4. [Output](#4-output)
5. [Environment and Configuration](#5-environment-and-configuration)
6. [Exit Behavior](#6-exit-behavior)
7. [Definition of Done](#7-definition-of-done)

---

## 1. Overview and Goals

### 1.1 Problem Statement

The coding agent library (`Session`) is programmable-first: it gives host applications full control over the agentic loop. But a library alone forces every user to write a host application before they can point an agent at a task. Developers need a zero-ceremony way to run a coding agent from the terminal, and CI systems need a headless way to invoke one from a script.

The CLI is the simplest possible host application for `Session`. It translates command-line arguments into a `SessionConfig`, subscribes to the event stream for rendering, and exits when the session completes. It does not add concepts of its own -- no workspace management, no persistent identity, no project-level config files. Every feature lives in the library; the CLI is the wiring.

### 1.2 Design Principles

**Thin wrapper.** The CLI configures a `Session` and renders its events. All intelligence -- tool execution, loop detection, truncation, subagent spawning -- lives in the library. The CLI never duplicates library logic.

**Dual-mode.** Interactive by default (streaming output, tool approval prompts). Fully scriptable with flags (`--auto-approve`). Same binary, same flags, different defaults based on whether a TTY is attached.

**Single command.** No subcommands. `agent <prompt>` is the only invocation. This keeps the mental model flat and the `--help` output short.

**Opinionated defaults.** The CLI picks reasonable defaults (provider, model, permissions) so that the common case requires zero flags. Power users override with explicit flags.

### 1.3 Relationship to Companion Specs

The CLI depends on the [Coding Agent Loop Specification](./coding-agent-loop-spec.md) for all agent behavior. It uses `Session`, `SessionConfig`, `EventEmitter`, `EventKind`, and `ExecutionEnvironment` directly. LLM communication flows through the agent library's use of the [Unified LLM Client Specification](./unified-llm-spec.md); the CLI does not interact with the LLM client directly.

```
┌─────────────────────────────┐
│  agent CLI                  │
│  (this spec)                │
│  - arg parsing              │
│  - event rendering          │
│  - permission prompts       │
└──────────┬──────────────────┘
           │ configures + runs
           ▼
┌─────────────────────────────┐
│  Session (agent library)    │
│  - agentic loop             │
│  - tools, subagents         │
│  - loop detection           │
└──────────┬──────────────────┘
           │ uses
           ▼
┌─────────────────────────────┐
│  LLM Client (llm library)  │
│  - provider adapters        │
│  - streaming, retry         │
└─────────────────────────────┘
```

---

## 2. Invocation

### 2.1 Usage

```
agent [OPTIONS] <PROMPT>
```

`PROMPT` is a required positional argument: the task for the agent to perform. No subcommands exist. No stdin reading, no REPL. If the user has a long prompt, they can quote it or use shell heredoc syntax.

### 2.2 Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--provider` | `String` | `"anthropic"` | LLM provider: `anthropic`, `openai`, or `gemini`. |
| `--model` | `String` | Provider default | Model identifier. When omitted, uses the provider profile's default model. |
| `--permissions` | `Enum` | `read-write` | Permission level: `read-only`, `read-write`, or `full`. See [Section 3](#3-permission-model). |
| `--auto-approve` | `Boolean` | `false` | Skip all interactive approval prompts. Denied tools are hard-blocked instead. |
| `--debug` | `Boolean` | `false` | Dump raw LLM request/response payloads to stderr. |

**No other flags.** The CLI does not expose: LLM parameters (temperature, max_tokens), subagent configuration, turn limits, session resumption, context injection, dry-run mode, structured output, working directory override, or verbose levels. These are deliberate omissions to keep the surface area minimal.

### 2.3 Provider and Model Resolution

The `--provider` flag selects the provider profile from the agent library:

| `--provider` value | Profile | Default model |
|---|---|---|
| `anthropic` | `AnthropicProfile` | Profile's default |
| `openai` | `OpenAiProfile` | Profile's default |
| `gemini` | `GeminiProfile` | Profile's default |

When `--model` is specified, it overrides the profile's default model but does not change the profile selection. The provider profile determines the system prompt, tool definitions, and tool-calling conventions.

**Why explicit --provider instead of auto-detection from model string.** Model naming conventions are not stable across providers and can collide. Explicit provider selection is unambiguous and avoids a mapping table that rots.

---

## 3. Permission Model

### 3.1 Permission Levels

The `--permissions` flag controls which tools the agent can use without approval:

| Level | Tools available without approval | Tools requiring approval |
|---|---|---|
| `read-only` | `read`, `grep`, `glob` | `write`, `edit`, `shell` |
| `read-write` | `read`, `grep`, `glob`, `write`, `edit` | `shell` |
| `full` | `read`, `grep`, `glob`, `write`, `edit`, `shell` | *(none)* |

The default is `read-write`: the agent can read and modify files freely but must ask before running shell commands.

### 3.2 Interactive Approval (TTY attached, no --auto-approve)

When the agent calls a tool that requires approval, the CLI prompts the user on stderr:

```
Agent wants to run shell: npm test
Allow? [y]es / [n]o / [a]lways
```

- **y**: Allow this single invocation. The agent proceeds. Future calls to the same tool still prompt.
- **n**: Deny this invocation. The agent receives a tool error: `"shell tool denied by user at current permission level"`. The agent must adapt.
- **a**: Escalate permissions for the remainder of the session. The tool (and all tools at or below its permission level) no longer prompt. Equivalent to upgrading `--permissions` mid-session.

### 3.3 Non-Interactive Mode (no TTY or --auto-approve)

When the CLI cannot prompt (piped stdin, `--auto-approve` set, or no TTY), tools that require approval are hard-blocked. The agent receives a tool error message and must find another way to accomplish the task.

`--auto-approve` does **not** implicitly upgrade to `--permissions full`. It means "don't prompt me, just enforce the stated permission level." A CI pipeline that wants full tool access must explicitly pass `--permissions full --auto-approve`.

**Why hard-block instead of auto-approve-all in CI.** Silent full access in CI is dangerous. Forcing `--permissions full` to be explicit makes the trust decision visible in the pipeline definition.

---

## 4. Output

### 4.1 Event Rendering

The CLI subscribes to the `Session`'s `EventEmitter` and renders events as follows:

| EventKind | Rendering |
|---|---|
| `AssistantMessage` | Stream text content to stdout as it arrives. |
| `ToolCall` | Print a one-line summary to stderr: tool name and key argument. |
| `ToolResult` | Suppressed (not shown to user). |
| `TurnComplete` | No output. |
| `Error` | Print error message to stderr. |

**Assistant text** streams to stdout character-by-character (or chunk-by-chunk as delivered by the LLM streaming response). This is the primary output.

**Tool call summaries** go to stderr so they don't interfere with piping stdout. Format:

```
[tool] read src/main.rs
[tool] edit src/lib.rs
[tool] shell npm test
```

### 4.2 Debug Mode

When `--debug` is set, the CLI additionally logs full LLM request and response payloads to stderr. This includes:

- The complete message array sent to the LLM
- System prompt
- Tool definitions
- Raw response body (streamed chunks or complete response)

Debug output is prefixed with `[debug]` to distinguish it from tool summaries.

### 4.3 Completion Summary

When the session ends, the CLI prints a one-line summary to stderr:

```
Done (4 turns, 7 tool calls, 3.2k tokens)
```

This always appears, regardless of whether the agent succeeded or failed. Token count is the total across all turns (input + output). The summary goes to stderr so stdout contains only the agent's text output.

---

## 5. Environment and Configuration

### 5.1 API Keys

API keys are read from standard environment variables. No config file, no `.env` loading, no key management.

| Provider | Environment variable |
|---|---|
| Anthropic | `ANTHROPIC_API_KEY` |
| OpenAI | `OPENAI_API_KEY` |
| Gemini | `GEMINI_API_KEY` |

If the required key is missing, the CLI exits immediately with a clear error message naming the expected variable.

### 5.2 Working Directory

The agent always operates in the process's current working directory. There is no `--dir` flag. Users who need a different directory use `cd` before invoking `agent`, following Unix convention.

### 5.3 Session Lifecycle

Every invocation is a fresh session. There is no session persistence, no resume flag, no checkpoint support at the CLI level. The agent starts, runs to completion, and exits. State between runs is carried only by the filesystem (files the agent created or modified).

---

## 6. Exit Behavior

### 6.1 Exit Codes

| Code | Meaning |
|---|---|
| `0` | Agent completed successfully. |
| `1` | Agent failed (LLM error, tool error, config error, agent gave up, or any other failure). |

Two codes only. The CLI does not differentiate between failure causes via exit code. Diagnostic information is in stderr output.

### 6.2 Interruption

Ctrl-C (SIGINT) triggers a graceful shutdown: the current LLM request is cancelled, any running tool is terminated, and the CLI exits with code 1. No cleanup prompt, no "are you sure" -- immediate stop.

---

## 7. Definition of Done

This section defines how to validate that an implementation of this spec is complete and correct. An implementation is done when every item is checked off.

### 7.1 Invocation

- [ ] `agent 'hello world'` sends "hello world" to the LLM and prints the response to stdout
- [ ] `agent` with no arguments prints usage and exits with code 1
- [ ] `--provider anthropic` uses the Anthropic profile
- [ ] `--provider openai` uses the OpenAI profile
- [ ] `--provider gemini` uses the Gemini profile
- [ ] `--model` overrides the default model within the selected profile
- [ ] Invalid `--provider` value exits with code 1 and a clear error

### 7.2 Permissions

- [ ] Default permission level is `read-write`
- [ ] `--permissions read-only` blocks write, edit, and shell
- [ ] `--permissions full` allows all tools without prompts
- [ ] In interactive mode, denied tools trigger an approval prompt on stderr
- [ ] Answering "y" allows a single invocation
- [ ] Answering "n" returns a tool error to the agent
- [ ] Answering "a" escalates permissions for the session
- [ ] In non-interactive mode, denied tools are hard-blocked (tool error returned to agent)
- [ ] `--auto-approve` does not implicitly upgrade permission level

### 7.3 Output

- [ ] Assistant text streams to stdout
- [ ] Tool call summaries print to stderr in `[tool] name args` format
- [ ] Tool results are not shown to the user
- [ ] Completion summary prints to stderr: `Done (N turns, N tool calls, Nk tokens)`
- [ ] `--debug` dumps full LLM request/response payloads to stderr

### 7.4 Configuration

- [ ] API keys read from `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GEMINI_API_KEY`
- [ ] Missing API key exits with code 1 and names the expected variable
- [ ] Agent operates in the current working directory

### 7.5 Exit Behavior

- [ ] Successful completion exits with code 0
- [ ] Any failure exits with code 1
- [ ] Ctrl-C cancels the current operation and exits with code 1

### 7.6 Integration Smoke Test

```
FUNCTION smoke_test():
    -- Setup
    SET dir = create_temp_directory()
    write_file(dir + "/hello.txt", "world")
    SET env = {"ANTHROPIC_API_KEY": valid_key}

    -- Test 1: Basic invocation
    SET result = run_cli(
        args: ["agent", "Read hello.txt and tell me what it says"],
        cwd: dir,
        env: env
    )
    ASSERT result.exit_code == 0
    ASSERT result.stdout CONTAINS "world"
    ASSERT result.stderr CONTAINS "Done ("
    ASSERT result.stderr CONTAINS "[tool] read"

    -- Test 2: Permission enforcement
    SET result = run_cli(
        args: ["agent", "--permissions", "read-only", "--auto-approve",
               "Write 'test' to output.txt"],
        cwd: dir,
        env: env
    )
    -- Agent should complete (exit 0) but output.txt should not exist
    -- because write was blocked and agent adapted
    ASSERT NOT file_exists(dir + "/output.txt")

    -- Test 3: Missing API key
    SET result = run_cli(
        args: ["agent", "hello"],
        cwd: dir,
        env: {}
    )
    ASSERT result.exit_code == 1
    ASSERT result.stderr CONTAINS "ANTHROPIC_API_KEY"

    -- Test 4: No arguments
    SET result = run_cli(
        args: ["agent"],
        cwd: dir,
        env: env
    )
    ASSERT result.exit_code == 1
```
