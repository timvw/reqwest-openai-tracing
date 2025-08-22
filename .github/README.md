# GitHub Configuration

## Rulesets

The `ruleset.json` file contains branch protection rules for this repository.

### To Import the Ruleset:

1. Go to **Settings** → **Rules** → **Rulesets** in your repository
2. Click **"New ruleset"** → **"Import a ruleset"**
3. Select the `ruleset.json` file from this directory
4. Review and click **"Create"**

### What the Ruleset Does:

- Requires pull requests for changes to main branch
- Requires status checks to pass (Test Suite)
- Requires branches to be up-to-date before merging
- Prevents force pushes and branch deletion
- Requires conversation resolution before merging

### Note:

Rulesets are a built-in GitHub feature (no external apps required) that allow version-controlled branch protection rules.