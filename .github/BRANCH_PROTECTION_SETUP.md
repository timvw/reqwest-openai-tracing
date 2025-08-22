# Branch Protection Setup

This repository includes configuration files for branch protection rules that can be version-controlled.

## Option 1: GitHub Rulesets (Recommended for GitHub Enterprise/Teams)

The `.github/ruleset.json` file contains the ruleset configuration. To import:

1. Go to Settings → Rules → Rulesets
2. Click "New ruleset" → "Import a ruleset"
3. Upload the `ruleset.json` file

## Option 2: GitHub Settings App (Recommended for Open Source)

The `.github/settings.yml` file works with the GitHub Settings app:

1. Install the app: https://github.com/apps/settings
2. Grant it access to your repository
3. The app will automatically sync settings from `.github/settings.yml`

The configuration will:
- Require all status checks to pass (Test Suite)
- Require PRs for all changes to main
- Require up-to-date branches before merging
- Prevent force pushes and deletions
- Auto-delete branches after merge

## Option 3: Manual Setup via GitHub UI

1. Navigate to: https://github.com/timvw/reqwest-openai-tracing/settings/branches
2. Click "Add branch protection rule"
3. Configure as follows:

### Branch name pattern
- `main`

### Protect matching branches

#### Require a pull request before merging
- [ ] Require approvals (optional for solo projects)
- [x] Dismiss stale pull request approvals when new commits are pushed
- [ ] Require review from CODEOWNERS (optional)

#### Require status checks to pass before merging
- [x] Require branches to be up to date before merging
- Select these required status checks:
  - `cargo test` (from "Test Suite" workflow)
  - `Code Coverage` (from "Test Suite" workflow)  
  - `Release-plz` (optional - from "Release-plz" workflow)

#### Other options (recommended)
- [x] Require conversation resolution before merging
- [x] Require linear history (optional - enforces clean history)
- [ ] Include administrators (check if you want rules to apply to yourself)
- [x] Restrict who can push to matching branches (optional)
- [ ] Allow force pushes (generally not recommended)
- [ ] Allow deletions (generally not recommended)

## Verification

After setup, PRs will show:
- Required checks that must pass
- A merge button that's disabled until all checks pass
- Clear status indicators for each required check

## Notes

- The exact status check names come from the job names in your workflows
- You can always bypass these rules as an admin if "Include administrators" is unchecked
- These rules help prevent accidental merges of broken code