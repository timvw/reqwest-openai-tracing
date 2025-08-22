# Repository Merge Settings

## Configure Squash-Only Merging

To ensure a clean, linear history with no merge commits:

### 1. Repository Settings (One-time setup)

Go to **Settings** → **General** → **Pull Requests** section and configure:

- ✅ **Allow squash merging** - Enabled
- ❌ **Allow merge commits** - Disabled  
- ❌ **Allow rebase merging** - Disabled (optional, can keep enabled)
- ✅ **Always suggest updating pull request branches**
- ✅ **Automatically delete head branches** - Enabled

### 2. Import the Ruleset

The `ruleset.json` includes:
- `required_linear_history` - Enforces a linear commit history
- `non_fast_forward` - Prevents force pushes

### 3. Result

With these settings:
- All PRs will be squashed into a single commit when merged
- No merge commits will appear in the main branch
- PR branches are automatically deleted after merge
- The main branch maintains a clean, linear history
- Each commit on main represents one complete feature/fix

### Why Squash Merging?

- **Clean History**: One commit per feature/fix
- **Easy Reverts**: Can revert entire features with one revert
- **Clear Changelog**: Each commit message describes a complete change
- **No Merge Commit Noise**: Linear history is easier to read and bisect