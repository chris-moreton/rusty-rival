# Agent instructions

## Optional rival-loop workflow

This repository can be worked on through the shared rival-loop multi-agent
workflow, but merely opening or working in this repository does **not** enrol an
agent in that workflow.

The rival-loop instructions apply only when the agent's launch prompt or the
user explicitly assigns it a rival-loop role (`manager`, `coder`, or
`reviewer`). A separately launched or otherwise unassigned agent should follow
its normal instructions and must not ask for a rival-loop role, announce a
rival-loop session name, contact the tmux sessions, or block ordinary work on
role assignment.

When a rival-loop role has been explicitly assigned, read and follow the shared
protocol exactly:

    /mnt/c/Users/chris/git/chris-moreton/my-claude-skills/skills/rival-loop/PROTOCOL.md

That protocol defines the roles, reporting relationships, message format,
worktree convention, and evidence standards. It is the single source of truth
for an active rival-loop session; do not copy or summarise it here. Changes to
the workflow itself belong in the `my-claude-skills` repository so every
participating agent receives them.
