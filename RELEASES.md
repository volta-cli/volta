# Version 0.3.0

- Support `lts` pseudo-version for Node (#331)
- Error message improvements
- Add `notion install` and `notion uninstall` for package binaries
- Remove autoshimming

# Version 0.2.2

- Add `notion which` command (#293)
- Show progress when fetching Notion installer (#279)
- Improved styling for usage information (#283)
- Support for `fish` shell (#266, #290)
- Consolidate binaries, for a ~2/3 size reduction of Notion installer (#274)

# Version 0.2.1

- Move preventing globals behind a feature flag (#273)

# Version 0.2.0

- Add support for OpenSSL 1.1.1 (#267)
- Fix: ensure temp files are on the same volume (#257)
- Intercept global package installations (#248)
- Fix: make npx compatible with prelrease versions of npm (#239)
- Fix: make `notion deactivate` work infallibly, without loading any files (#237)
- Fix: make `"npm"` key optional in `package.json` (#233)
- Fix: publish latest Notion version via self-hosted endpoint (#230)
- Fix: eliminate excessive fetching and scanning for exact versions (#227)
- Rename `notion use` to `notion pin` (#226)
- Base filesystem isolation on `NOTION_HOME` env var (#224)
- Fix: robust progress bar logic (#221)
- Use JSON for internal state files (#220)
- Support for npm and npx (#205)
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
