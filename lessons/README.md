# Lessons

- [modal-00-baseline.toml](modal-00-baseline.toml) is the shipped, fully-calibrated modal foundations lesson covering mode entry/exit (`i`, `a`, `<Esc>`, `ctrl-[`), directional navigation (`h`/`j`/`k`/`l`), and inline diagnostic error jumping (`g ]`/`g [`).
- [motions-01-word.toml](motions-01-word.toml) is the shipped, fully-calibrated starter course lesson covering word motions (`dw`, `de`, `w` + `dw`).
- [motions-02-find.toml](motions-02-find.toml) is the shipped, fully-calibrated find-motions lesson covering intra-line delimiter navigation and operators (`df`, `dt`, `ct`, `;`, `,`).
- [motions-03-textobjects.toml](motions-03-textobjects.toml) is the shipped, fully-calibrated text-objects lesson covering delimited text objects (`ci"`, `ci{`, `ci(`) and Zed agnostic quotes/brackets, admitted under [ZK-025.3](../docs/BACKLOG.md#task-zk-025-3).
- [operators-01-edit.toml](operators-01-edit.toml) is the shipped, fully-calibrated core operators lesson covering character replacement (`r`), count multipliers (`2dw`), linewise indentation (`>>`), native surrounds (`cs`, `ds`, `ys`), and keymap contexts (`VimControl`, `vim_mode == normal`, `dev: open key context view`), admitted under [ZK-026.1](../docs/BACKLOG.md#task-zk-026-1), [ZK-026.2](../docs/BACKLOG.md#task-zk-026-2), and [ZK-026.3](../docs/BACKLOG.md#task-zk-026-3).
- [zed-01-lsp-actions.toml](zed-01-lsp-actions.toml) is the normal-mode code intelligence and language server actions module covering symbol rename (`c d`), code actions (`g .`), diagnostic navigation (`g ]`/`g [` and `:clist`), definition jumping (`g d`, `<ctrl-w> g d`), and hover documentation (`g h`), added under [ZK-027.1](../docs/BACKLOG.md#task-zk-027-1).

For the full curriculum sequence, prerequisites DAG, and capability audit, see:
- [Curriculum Module Map](../docs/curriculum-module-map.md)
- [Curriculum Author Instructions](../.claude/agents/curriculum-author.md)


