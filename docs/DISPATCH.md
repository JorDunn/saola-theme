# Dispatcher runbook

How the main Claude session runs the board for this repo. Written for a
session with no memory of earlier ones: read this and `CLAUDE.md`, then start
at [The loop](#the-loop).

This is the saola-theme counterpart of `docs/DISPATCH.md` in
`JorDunn/saola-files`. The two share the board, the label names and the
branch and PR rules; this file adds the release step, because here a consumer
is unblocked by a tag, not by a merge.

## Where the work lives

-   Board: the "Saola" user Project,
    <https://github.com/users/JorDunn/projects/2> (project number `2`, owner
    `JorDunn`). Never create a second project.
-   Repos on the board: `JorDunn/saola-theme` (this one) and consumer repos
    such as `JorDunn/saola-files`. The dispatcher here implements saola-theme
    tasks only; each consumer repo is worked by the session named after it.
-   Epics are milestones with sub-issues (milestone number order is work
    order). Ordering between tasks is GitHub's "blocked by" link.
-   The Project's `Status` field: `Todo`, `In Progress`, `Done`.
-   `PLAN.md` is retired and gitignored. Do not recreate it.

## Labels

| Label           | Meaning                                                        |
| --------------- | -------------------------------------------------------------- |
| `theme-request` | A consumer repo needs a token, style, glyph or style-guide exception. Stays open until the release tag exists. |
| `area:*`        | Part of the code the task touches (`tokens`, `style`, `icons`, `gallery`, `export`, `docs`, `ci`). Informational. |
| `model:sonnet`  | Implement with Sonnet. The default when no `model:*` label is set. |
| `model:opus`    | Implement with Opus.                                           |
| `solo`          | Touches most of the tree; runs with no other task in flight.   |
| `lock:tokens`   | Changes a token struct, `Theme::saola()` or `themes/saola.toml`; never concurrent with another holder. |
| `lock:icons`    | Changes the `Icon` enum or its assets; never concurrent with another holder. |
| `needs-jordan`  | Waits on the maintainer. Never dispatched.                     |
| `epic`          | Parent issue that groups sub-issues. Never dispatched.         |
| `bug`           | Defect, including a failed visual check.                       |

`themes/saola.toml` is regenerated whole from `Theme::saola()`, so two
branches that each add a token always conflict there. That is what
`lock:tokens` prevents.

## Incoming theme requests

A consumer session files the issue, adds it to the board, links it as a
blocker of its own task, and sends one message per batch. The request is
input, not a specification: check it before it becomes work.

1.  Read the issue against `CLAUDE.md` ("Design language" and "Boundaries")
    and the existing API. If the crate already covers the need, say so on
    the issue and close it.
2.  Label it `needs-jordan` and do not dispatch it when it needs any of:
    a fourth colour or a new style-guide exception; a token value with no
    design source; a new dependency; a change to the "Design language" or
    "Boundaries" sections of `CLAUDE.md`.
3.  Otherwise add the `area:*`, `model:*` and `lock:*` labels it needs. If
    one request holds several independent pieces, split it into sub-issues
    and treat the request as their parent.

## Ready queue

A task is ready when all of these hold:

-   it is open, in `JorDunn/saola-theme`;
-   every `blockedBy` issue is `CLOSED` (cross-repo blockers included);
-   it has no assignee (assigned = in flight);
-   it is not an epic: `subIssuesSummary.total == 0` and no `epic` label;
-   it is not labelled `needs-jordan`.

Order: `theme-request` issues first (a consumer is waiting), then lowest
milestone number, then lowest issue number.

```bash
gh api graphql --paginate --slurp -f owner=JorDunn -f repo=saola-theme -f query='
query($owner: String!, $repo: String!, $endCursor: String) {
  repository(owner: $owner, name: $repo) {
    issues(states: OPEN, first: 100, after: $endCursor) {
      pageInfo { hasNextPage endCursor }
      nodes {
        number
        title
        milestone { number title }
        assignees(first: 5) { nodes { login } }
        labels(first: 30) { nodes { name } }
        subIssuesSummary { total }
        blockedBy(first: 50) { nodes { state number repository { nameWithOwner } } }
      }
    }
  }
}' | jq -r '
  [.[].data.repository.issues.nodes[]]
  | map(select(
      (.blockedBy.nodes | all(.state == "CLOSED"))
      and (.assignees.nodes | length == 0)
      and (.subIssuesSummary.total == 0)
      and ([.labels.nodes[].name]
           | any(. == "needs-jordan" or . == "epic")
           | not)))
  | sort_by(
      ([.labels.nodes[].name] | any(. == "theme-request") | not),
      (.milestone.number // 999999),
      .number)
  | .[]
  | "#\(.number)\t\(.milestone.title // "-")\t\([.labels.nodes[].name] | join(","))\t\(.title)"'
```

Output: one line per ready task, `#<n>  <milestone>  <labels>  <title>`.

A `theme-request` whose PR has merged but whose tag does not exist yet is
still open and still assigned, so the query does not return it again.

## Concurrency

-   At most 3 tasks in flight.
-   A `solo` task starts only when nothing is in flight, and nothing else
    starts until it merges or stops.
-   Two tasks carrying the same `lock:*` label never run together.
-   When the queue holds more ready tasks than free slots, take them in queue
    order, skipping any that would break a rule above.

## The loop

Repeat until a [stop condition](#stop-conditions) holds:

1.  Run the ready query.
2.  Fill free slots under the concurrency rules (lifecycle below).
3.  As PRs go green and pass review, merge them and run their visual checks.
4.  Re-run the query after every merge; a merge can unblock tasks.
5.  When `main` holds an unreleased `theme-request` change, run the
    [release step](#release).

## Task lifecycle

### 1. Claim

```bash
gh issue edit <n> -R JorDunn/saola-theme --add-assignee @me
```

Set Status to `In Progress`. Field and option IDs can change, so look them
up each session rather than hard-coding them:

```bash
gh project view 2 --owner JorDunn --format json --jq .id      # project id
gh project field-list 2 --owner JorDunn --format json         # Status field id + option ids
gh api graphql -f query='{repository(owner:"JorDunn",name:"saola-theme"){
  issue(number:<n>){projectItems(first:5){nodes{id project{number}}}}}}'  # item id
gh project item-edit --project-id <project-id> --id <item-id> \
  --field-id <status-field-id> --single-select-option-id <in-progress-id>
```

### 2. Implement

Spawn one implementation subagent per task, with `isolation: "worktree"`,
on the model from the task's `model:*` label (Sonnet if none). A change of a
few lines the dispatcher fully understands may be done in the main session
instead, on the same branch and PR rules. The brief contains:

-   the issue number, title and full body;
-   the "notes for the next task" from the PRs that closed this task's
    blockers;
-   branch `task/<n>-<slug>` from the latest `origin/main`;
-   "read `CLAUDE.md` first; it is binding" and "use the
    `test-driven-development` skill";
-   the local gate, all of which must pass before pushing:

    ```bash
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo build -p saola-theme --example gallery
    cargo tree -p saola-tokens -e normal   # only serde/toml/thiserror
    ```

-   PR rules: title is the Conventional Commit (it becomes the squash commit
    and the changelog line); body follows the PR template; any
    `BREAKING CHANGE:` footer goes in the body. The body says `Closes #<n>`
    for an ordinary task and **`Refs #<n>` for a `theme-request`**, which
    must stay open until its tag exists;
-   "do not merge; do not touch other issues; do not bump a version or edit
    a CHANGELOG.md"; if a value has no design source, stop and report it
    rather than inventing one;
-   the report to return: PR URL, gate results, new public API (paths and
    signatures), gallery specimens to check, anything unfinished.

### 3. Review

Spawn a fresh review subagent (never the author) to run `/code-review` on
the PR against its issue. Send findings to the author agent to fix, then
review again. After two rounds without a clean review, label the PR and
issue `needs-jordan`, unassign the slot, and move on.

### 4. Merge

On a clean review and green required checks (`fmt`, `clippy`, `test`,
`gallery`, `pr-title`):

```bash
gh pr merge <pr> --squash --auto
```

If auto-merge is unavailable, wait for the checks and merge directly:

```bash
gh pr checks <pr> --watch --required --fail-fast && gh pr merge <pr> --squash
```

If the PR is behind `main` and checks need a re-run, `gh pr update-branch
<pr>` first. The squash commit takes the PR title and body. An ordinary task
closes through `Closes #<n>`; set its Status to `Done` if the board did not.
A `theme-request` stays open and `In Progress` until the release step.

### 5. Visual check

After merge, the dispatcher alone runs `cargo run --example gallery` on an
up-to-date `main` (it needs a Wayland or X11 session) and looks at the PR's
specimens on both surfaces. Screenshots stay in the local scratchpad; never
attach them to GitHub (public repo). Write a worded result as a comment on
the issue. If no graphical session is available, say so in that comment
instead of skipping silently.

If the check fails, file a `bug` task describing the failure in words. A
failed check on a `theme-request` holds the release step until the bug task
merges.

## Release

Consumers pin saola-theme to a release tag, so a request is delivered when
the tag exists, not when the PR merges.

1.  Every push to `main` makes release-plz open or update the
    `chore: release` PR (per-crate version bumps and changelog entries).
    Never edit that PR's content by hand.
2.  When `main` holds an unreleased `theme-request` change and the release
    PR's required checks are green, the dispatcher squash-merges it
    (maintainer decision, 2026-11-19). Do not wait to batch further requests.

    ```bash
    gh pr list -R JorDunn/saola-theme --search 'chore: release in:title' --state open
    gh pr checks <pr> --watch --required --fail-fast && gh pr merge <pr> --squash
    ```

3.  The release job then pushes the tags. Confirm them; a tag is the only
    proof of a release:

    ```bash
    git fetch --tags origin && git tag --sort=-creatordate | head -5
    ```

4.  For each `theme-request` the tag ships: comment the tag and the new
    public API on the issue, close it, and set Status to `Done`. Closing is
    the signal that unblocks the consumer's task.
5.  Send one message per released batch to the requesting session: the tag,
    the closed issues, and anything it must change on the pin bump.

A release that carries no `theme-request` change (docs, CI, internal
refactors) is not urgent; merge its release PR with the next one that is, or
when the maintainer asks.

## Always `needs-jordan`

Never dispatched until the maintainer decides:

-   a fourth colour, or any new style-guide exception;
-   a token value with no design source;
-   a new dependency in any crate, and any GUI dependency in `saola-tokens`;
-   any change to the "Design language" or "Boundaries" sections of
    `CLAUDE.md`.

## Stop conditions

Stop the loop when any of these holds:

-   nothing is ready, nothing is in flight and no release is pending;
-   CI on `main` fails twice running;
-   only `needs-jordan` items remain.

The stop report contains:

-   merged PRs, each with its issue and one-line result;
-   tags cut, each with the requests it closed and the session told;
-   blocked items and what each waits on (issue, maintainer decision);
-   visual-check results, worded, including any `bug` tasks filed;
-   anything left in flight and its state.
