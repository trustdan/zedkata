# Repository organization

Scope: the user's requested repository cleanup, following the folder layout already
referenced by the existing documents.

1. Move architecture, backlog, and open questions into `docs/` and decisions into
   `docs/adr/`.
2. Move specialist agent definitions into `.claude/agents/` and the reference lesson
   into `lessons/`. Keep `README.md` and `CLAUDE.md` at the root.
3. Add navigation and explain the sample lesson's incomplete status. Fix its agent
   instruction path.
4. Check that every original file survives, moved content is preserved, and local
   Markdown links resolve.

This cleanup does not advance the project beyond its Phase 0 feasibility gate.
