# Version 0.2.0

- Ensure temp files are on the same volume (#257)
- Intercept global package installations (#248)
- Make `notion deactivate` work infallibly, without loading any files (#237)
- Make `"npm"` key optional in `package.json` (#233)
- Publish latest Notion version via self-hosted endpoint (#230)
- Eliminate excessive fetching and scanning for exact versions (#227)
- Rename `notion use` to `notion pin` (#226)
- Base filesystem isolation on `NOTION_HOME` env var (#224)
- Robust progress bar logic (#221)
- Use JSON for internal state files (#220)
- Support for npm and npx (#205)
- Smoke tests (#188)
- Changes to directory layout (#181)

# Version 0.1.5

- Autoshimming! (#163)
- `notion deactivate` also unsets `NOTION_HOME` (#195)
- Implemented `notion activate` (#201)
- Fix for Yarn over-fetching bug (#203)

# Version 0.1.4

- Fix for `package.json` parsing bug (#156)

# Version 0.1.3

- Fix for Yarn path bug (#153)

# Version 0.1.2

- Correct logic for computing `latest` version of Node (#144)
- Don't crash if cache dir was deleted (#138)
- Improved tests (#135)

# Version 0.1.1

- Support for specifying `latest` as a version specifier (#133)
- Suppress scary-looking symlink warnings on reinstall (#132)
- Clearer error message for not-yet-implemented `notion install somebin` (#131)
- Support optional `v` prefix to version specifiers (#130)

# Version 0.1.0

First pre-release, supporting:

- macOS and Linux (bash-only)
- `notion install` (Node and Yarn only, no package binaries)
- `notion use`
- Proof-of-concept plugin API
