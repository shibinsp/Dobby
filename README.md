# Dobby CLI

Dobby CLI is a Forge-compatible workflow assistant implemented entirely in Rust. It layers focused planning and task management commands on top of the upstream Forge experience, so you can capture implementation intent and still access every Forge feature from a single binary.

## Quickstart
Install or update the CLI with a single command (mirroring Forge's installer):

```bash
curl -fsSL https://raw.githubusercontent.com/shibinsp/Dobby/main/scripts/install.sh | bash
```

This script checks for Rust tooling, fetches the latest `main` branch by default, and installs the `dobby` binary into `~/.cargo/bin`. You can pin a release or branch with environment overrides, e.g. `DOBBY_TAG=v0.1.0 curl ... | bash` (alias: `DOBBY_VERSION`).

## Key capabilities
- **Plan scaffolding** – capture a project name, description, and milestones with `dobby plan init`, then inspect the live blueprint with `dobby plan show`.
- **Task tracking** – add work items, filter by status, and update progress via human-friendly IDs or simple list indexes.
- **Stateful workflows** – plan and task data lives in `~/.dobby-cli/state.json`, so every session resumes from the same source of truth until you reset it.
- **Forge delegation** – any command outside of `plan`/`task` is forwarded to the vendored Forge binary, exposing 100% of the upstream CLI (agents, providers, workspaces, conversations, etc.).

## Installation
Prefer the single-command installer above? Skip to [Quickstart](#quickstart). For manual control, build the CLI directly from this repository.

### Prerequisites
- A Rust toolchain via [`rustup`](https://rustup.rs/)
- Protocol Buffers (`brew install protobuf` on macOS or download from the official releases page)
- Git submodules initialized (`git submodule update --init --recursive`)

### Install once and reuse
```bash
cargo install --path .
```
This places the `dobby` binary in `~/.cargo/bin`, making it available anywhere on your system.

### Run ad-hoc without installing
```bash
cargo run -- plan show
```

The compiled binary lives at `target/release/dobby` (or `target/debug/dobby` when using `cargo run`).

## Forge integration
Dobby vendors the entire [antinomyhq/forgecode](https://github.com/antinomyhq/forgecode) tree under `vendor/forgecode` and delegates every non-`plan`/`task` command to the real Forge binary. To keep the delegation path working:

1. Pull the submodule after cloning: `git submodule update --init --recursive`
2. Install prerequisites: a Rust toolchain (`rustup`) and Protocol Buffers (`brew install protobuf` on macOS, or download from https://github.com/protocolbuffers/protobuf/releases).
3. Compile Forge once with `cargo build --release --manifest-path vendor/forgecode/Cargo.toml` (or let `dobby` build it automatically the first time you run a delegated command).
4. Invoke any Forge command through Dobby, e.g. `dobby provider list` or `dobby` for the interactive shell. Dobby-native workflows (`plan`/`task`) continue to run directly inside this binary.

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

### Delegated Forge commands
Any other invocation is proxied to the Forge binary. For example:

```bash
# Launch Forge's interactive shell
dobby

# List configured providers via Forge
dobby provider list
```

## Development workflow
1. Install dependencies: `rustup target add` (as needed) and `cargo fetch`
2. Format & lint: `cargo fmt && cargo clippy`
3. Compile: `cargo build`
4. Exercise flows locally: `cargo run -- plan show` or `cargo run -- provider list`

## Testing
Run the standard Rust checks before shipping changes:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Validate end-to-end behavior by running representative commands (swap `cargo run --` with the installed `dobby` binary if you've already installed it):

```bash
cargo run -- plan init -n "Demo" -m "First milestone"
cargo run -- task add "Wire up persistence"
cargo run -- task list
```

## Troubleshooting
- **Plan already exists:** Reset with `dobby plan reset` before re-running `plan init`.
- **`No task matches identifier` errors:** Double-check the task ID or use the list index when updating statuses.
- **Forge build failures:** Confirm Rust (`cargo`) and Protocol Buffers (`protoc`) are installed, then rerun `git submodule update --init --recursive` followed by `cargo build --release --manifest-path vendor/forgecode/Cargo.toml`.
