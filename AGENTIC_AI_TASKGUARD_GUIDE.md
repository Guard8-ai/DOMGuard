# TaskGuard for AI Agents

## Quick Reference

```bash
# Core commands
taskguard init                                    # Initialize
taskguard create --title "Task" --area backend \
  --dependencies "setup-001"                      # Create (deps REQUIRED)
taskguard list                                    # List tasks
taskguard validate                                # Check dependencies
taskguard validate --orphans                      # Check orphan tasks
taskguard update status <id> doing                # Update status

# Full create (ALWAYS use --dependencies)
taskguard create --title "Task" --area backend --priority high \
  --complexity 7 --dependencies "setup-001" --estimate "4h"

# Orphan escape hatch (not recommended)
taskguard create --title "Spike" --allow-orphan-task

# Update commands (format: update <field> <id> <value>)
taskguard update status <id> doing
taskguard update status <id> done
taskguard update priority <id> high
taskguard update dependencies <id> "dep1,dep2"
taskguard update assignee <id> "name"

# Checklist items
taskguard list items <id>                         # View checklist
taskguard task update <id> <item#> done           # Mark item done

# GitHub sync
taskguard sync --github                           # Sync to GitHub
taskguard archive                                 # Archive done tasks
taskguard restore <id>                            # Restore archived
```

## Causality Tracking (v0.4.0+, improved in v0.4.2)

**Every task MUST have dependencies.** `setup-001` is auto-created by `taskguard init`.

```
setup-001 → backend-001 → api-001 → testing-001
         ↘ frontend-001 → integration-001
```

If CAUTION appears: `taskguard update dependencies <id> "parent-id"`

## Workflow

1. **Init**: `taskguard init && taskguard validate`
2. **Create**: `--dependencies "setup-001"` or `"previous-task-id"`
3. **Start**: Read dependency task files first
4. **Validate**: `taskguard validate` after each change
5. **Update**: Use CLI commands, not manual file editing
6. **Complete**: `taskguard update status <id> done`
7. **Commit**: `git add -A && git commit -m "feat(area): description"`

## Areas

`setup` | `backend` | `api` | `frontend` | `auth` | `data` | `testing` | `deployment` | `docs` | `integration`

## Priority

`critical` > `high` > `medium` > `low`

## Status Flow

`todo` → `doing` → `review` → `done` (or `blocked`)

## Common Mistakes

| Wrong | Right |
|-------|-------|
| No dependencies | **ALWAYS use `--dependencies`** |
| Ignore CAUTION | Fix orphans immediately |
| Manual YAML editing | Use CLI commands |
| No validation | `taskguard validate` frequently |

## Troubleshooting

```bash
taskguard validate --orphans # See orphan tasks
taskguard list --area X      # Filter by area
gh auth status               # Check GitHub auth
```

---
**Rule**: Use CLI for metadata. Always use `--dependencies`. No orphans.
