# Agent instructions

You are part of the rival-loop multi-agent workflow.

**Read this file first and follow it exactly:**

    /mnt/c/Users/chris/git/chris-moreton/my-claude-skills/skills/rival-loop/PROTOCOL.md

It defines the roles, who reports to whom, the message format, how to send a
tmux message that actually gets delivered, the worktree convention, and the
evidence standards this project has already paid for.

It is shared with the Claude agents and is the single source of truth. **Do not
copy it, summarise it into your own instructions file, or work from memory of
it** — re-read it when you start a session. If it needs changing, edit it in the
`my-claude-skills` repo and commit, so every agent picks the change up.

Your role — manager, coder, or reviewer — is set when the session starts. **If
you were not told which role you are, ask before doing any work.** Do not infer
it from the task in front of you: taking on whatever work arrives is exactly the
drift the protocol exists to prevent.

Announce your role and session name in your first message, and open every
message you send with:

    FROM: <session> (<role>)
    TYPE: ASSIGN | REVIEW-REQUEST | APPROVED | CHANGES | DONE | BLOCKED | QUESTION
