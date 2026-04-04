# Dobby CLI

Dobby CLI is a Forge-inspired workflow assistant that helps you capture implementation plans and track coding tasks without leaving the terminal. Plans and tasks are stored locally so you can pick up where you left off across sessions.

## Key capabilities
- **Plan scaffolding** – capture a project name, description, and milestones with `dobby plan init`, then review the current blueprint with `dobby plan show`.
- **Task tracking** – add work items, filter by status, and update progress via human-friendly IDs or simple list indexes.
- **Stateful workflows** – data lives in `~/.dobby-cli/state.json`, so every command builds on the same source of truth until you reset it.

## Installation
```bash
npm install -g dobby-cli
```

Or run it ad-hoc without a global install:
```bash
npx dobby-cli plan show
```

## Forge integration
Dobby now vendors the entire [antinomyhq/forgecode](https://github.com/antinomyhq/forgecode) tree under `vendor/forgecode` and delegates every command outside of `plan`/`task` to the real Forge binary. This gives you immediate access to commands such as `provider login`, `workspace sync`, `conversation list`, etc., without re-implementing them in TypeScript.

1. Pull the submodule after cloning: `git submodule update --init --recursive`
2. Install the prerequisites: a Rust toolchain (`cargo`) and Protocol Buffers (`brew install protobuf` on macOS, or download from https://github.com/protocolbuffers/protobuf/releases).
3. Compile Forge once with `npm run forge:build` (or let `dobby` build it automatically the first time you run a delegated command).
4. Run any Forge command through Dobby, for example `dobby provider list` or `dobby` for the interactive shell. Dobby-native workflows (`plan`/`task`) continue to work as before.

## Usage

### Plan commands
| Command | Description |
| --- | --- |
| `dobby plan init -n "Feature" -d "Optional description" -m "First milestone" -m "Second milestone"` | Create a new plan (fails if one already exists). |
| `dobby plan show` | Display the active plan with milestones and color-coded tasks (IDs are shown for easy cross-reference). |
| `dobby plan reset` | Clear the stored plan/tasks so you can start over. |

### Task commands
| Command | Description |
| --- | --- |
| `dobby task add "Implement auth" --notes "Start with backend"` | Append a pending task to the active plan. |
| `dobby task list` | List every task. Add `--status in_progress` (or `pending`, `completed`) to filter. |
| `dobby task status 2 completed` | Update a task by 1-based index. You can also pass a full ID or a unique ID prefix (e.g., `dobby task status a1b2c3 in_progress`). |

> **Tip:** Run `dobby plan show` or `dobby task list` to grab the colored IDs shown next to each task before updating their status.

## Development workflow
1. Install dependencies: `npm install`
2. Start iterating with live reload: `npm run dev`
3. Compile once you're ready to distribute: `npm run build`

## Troubleshooting
- **Plan already exists:** Reset with `dobby plan reset` before re-running `plan init`.
- **"No task matched" errors:** Double-check the task ID or use the list index when updating statuses.
