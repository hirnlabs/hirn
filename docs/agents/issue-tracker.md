# Issue tracker: Linear

Issues and PRDs for this repo live in Linear. Use the `linear` MCP server for all operations.

## Projects and Subfolders

This repository is multi-context. For each subfolder/context in this repo, there is a matching project in Linear. **Always assign issues to their corresponding project**:

- `agent` subfolder → **agent** project
- `assistant` subfolder → **assistant** project
- `data` subfolder → **data** project
- `desktop` subfolder → **desktop** project
- `homepage` subfolder → **homepage** project
- `router` subfolder → **router** project
- `sdk` subfolder → **sdk** project
- `server` subfolder → **server** project
- `transcribe` subfolder → **transcribe** project

When creating or modifying an issue that affects a specific subfolder/context, specify the matching project name in the `project` argument of the `linear:save_issue` call.

## Conventions

- **Create an issue**: Call `linear:save_issue` with `title`, `description`, `team`, and `project`.
- **Read an issue**: Call `linear:get_issue` with `id` (e.g., `HIR-1`).
- **List issues**: Call `linear:list_issues` with `team` and optionally `project` filters.
- **Comment on an issue**: Call `linear:save_comment` with `issueId` and `body`.
- **Apply/remove labels or update status**: Call `linear:save_issue` with `id` and the parameters to update (e.g. `labels`, `state`).
- **Close an issue**: Call `linear:save_issue` with `id` and set `state` to `Done`.

## When a skill says "publish to the issue tracker"

Create a Linear issue using `linear:save_issue`. Make sure to assign it to the correct project based on the context/subfolder it relates to.

## When a skill says "fetch the relevant ticket"

Call `linear:get_issue` with the issue key (e.g., `HIR-1`).
