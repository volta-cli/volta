# Version 2.0.0

- 🚨 (BREAKING) 🚨 We upgraded the version of Rust used to build Volta, which drops support for older versions of glibc & Linux kernel. See [the Rust announcement from August 2022](https://blog.rust-lang.org/2022/08/01/Increasing-glibc-kernel-requirements.html) for details about the supported versions. Notably, this means that we no longer support CentOS 6 (#1611)
- 🚨 (BREAKING) 🚨 Due to costs and changes in the code signing process, we have dropped the code signing for the Windows installer. We now recommend using `winget` to install Volta on Windows (#1650)
- 🎉 (NEW) 🎉 We now ship a pre-built binary for ARM Linux & ARM Windows (#1696, #1801)
- Volta no longer requires Developer Mode to be enabled on Windows (#1755)
- `volta uninstall` now provides better help & error messages to describe its use and limitations (#1628, #1786)
- Volta will now use a universal binary on Mac, rather than separate Intel- & ARM-specific builds (#1635)
- Switched to installing profile scripts into `.zshenv` by default, rather than `.zshrc` (#1657)
- Added a default shim for the `yarnpkg` command, which is an alias of `yarn` (#1670)
- Added a new `--very-verbose` flag to enable even more logging (note: we haven't yet implemented much additional logging) (#1815)
- Simplified the fetching process to remove an extra network request and resolve hangs (#1812)
- Several dependency upgrades and clean-up refactors from @tottoto

# Version 1.1.1

- Experimental support for pnpm (requires `VOLTA_FEATURE_PNPM` environment variable) (#1273)
- Fix to correctly import native root certificates (#1375)
- Better detection of executables provided by `yarn` (#1388, #1393)

# Version 1.1.0

- Added support for pinning / installing Yarn 3+ (#1305)
- Improved portability and installer effectiveness by removing dependency on OpenSSL (#1214)

# Version 1.0.8

- Fix for malformed `bin` entries when installing global packages (#997)
- Dependency updates

# Version 1.0.7

- Added build for Linux distros with OpenSSL 3.0 (#1211)

# Version 1.0.6

- Fixed panic when `stdout` is closed (#1058)
- Disabled global package interception when `--prefix` is provided (#1171)
- Numerous dependency updates

# Version 1.0.5

- Added error when attempting to install Node using `nvm` syntax (#1020)
- Avoid modifying shell config if the environment is already correct (#990)
- Prevent trying to read OS-generated files as package configs (#981)

# Version 1.0.4

- Fetch native Apple silicon versions of Node when available (#974)

# Version 1.0.3

- Fix pinning of `npm@bundled` when there is a custom default npm version (#957)
- Use correct binary name for scoped packages with a string `bin` entry in `package.json` (#969)

# Version 1.0.2

- Fix issues where `volta list` wasn't showing the correct information in all cases (#778, #926)
- Make detection of tool name case-insensitive on Windows (#941)
- Fix problem with `npm link` in a scoped package under npm 7 (#945)

# Version 1.0.1

- Create Native build for Apple Silicon machines (#915, #917)

# Version 1.0.0

- Support for `npm link` (#888, #889, #891)
- Support for `npm update -g` and `yarn global upgrade` (#895)
- Improvements in the handling of `npm` and `yarn` commands (#886, #887)

# Version 0.9.3

- Various fixes to event plugin logic (#892, #894, #897)

# Version 0.9.2

- Correctly detect Volta binary installation directory (#864)

# Version 0.9.1

- Fix an issue with installing globals using npm 7 (#858)

# Version 0.9.0

- Support Proxies through environment variables (#809, #851)
- Avoid unnecessary `exists` calls for files (#834)
- Rework package installs to allow for directly calling package manager (#848, #849)
- **Breaking Change**: Remove support for `packages` hooks (#817)

# Version 0.8.7

- Support fetching older versions of Yarn (#771)
- Correctly detect `zsh` environment with `ZDOTDIR` variable (#799)
- Prevent race conditions when installing tools (#684, #796)

# Version 0.8.6

- Improve parsing of `engines` when installing a package (#791, #792)

# Version 0.8.5

- Improve the stability of installing tools on systems with virus scanning software (#784)
- Make `volta uninstall` work correctly when the original install had an issue (#787)

# Version 0.8.4

- Add `{{filename}}` and `{{ext}}` (extension) replacements for `template` hooks (#774)
- Show better error when running `volta install yarn` without a Node version available (#763)

# Version 0.8.3

- Fix bug preventing custom `npm` versions from launching on Windows (#777)
- Fix for completions in `zsh` for `volta list` (#772)

# Version 0.8.2

- Add support for workspaces through the `extends` key in `package.json` (#755)
- Improve `volta setup` to make profile scripts more shareable across machines (#756)

# Version 0.8.1

- Fix panic when running `volta completions zsh` (#746)
- Improve startup latency by reducing binary size (#732, #733, #734, #735)

# Version 0.8.0

- Support for pinning / installing custom versions of `npm` (#691)
- New command: `volta run` which will let you run one-off commands using custom versions of Node / Yarn / npm (#713)
- Added default pretty formatter for `volta list` (#697)
- Improved setup of Volta environment to make it work in more scenarios (#666, #725)
- Bug fixes and performance improvements (#683, #701, #703, #704, #707, #717)

# Version 0.7.2

- Added `npm.cmd`, `npx.cmd`, and `yarn.cmd` on Windows to support tools that look for CMD files specifically (#663)
- Updated `volta setup` to also ensure that the shim symlinks are set up correctly (#662)

# Version 0.7.1

- Added warning when attempting to `volta uninstall` a package you don't have installed (#638)
- Added informational message about pinned project version when running `volta install` (#646)
- `volta completions` will attempt to create the output directory if it doesn't exist (#647)
- `volta install` will correctly handle script files that have CRLF as the line ending (#644)

# Version 0.7.0

- Removed deprecated commands `volta activate`, `volta deactivate`, and `volta current` (#620, #559)
- Simplified installer behavior and added data directory migration support (#619)
- Removed reliance on UNC paths when executing node scripts (#637)

# Version 0.6.8

- You can now use tagged versions when installing a tool with `volta install` (#604)
- `volta install <tool>` will now prefer LTS Node when pinning a version (#604)

# Version 0.6.7

- `volta pin` will no longer remove a closing newline from `package.json` (#603)
- New environment variable `VOLTA_BYPASS` will allow you to temporarily disable Volta shims (#603)

# Version 0.6.6

- Node and Yarn can now both be pinned in the same command `volta pin node yarn` (#593)
- Windows installer will now work on minimal Windows installs (e.g. Windows Sandbox) (#592)

# Version 0.6.5

- `volta list` Now always outputs to stdout, regardless of how it is called (#581)
- DEPRECATION: `volta activate` and `volta deactivate` are deprecated and will be removed in a future version (#571)

# Version 0.6.4

- `volta install` now works for installing packages from a private, authenticated registry (#554)
- `volta install` now has better diagnostic messages when things go wrong (#548)

# Version 0.6.3

- `volta install` will no longer error when installing a scoped binary package (#537)

# Version 0.6.2

- Added `volta list` command for inspecting the available tools and versions (#461)

# Version 0.6.1

- Windows users will see a spinner instead of a � when Volta is loading data (#511)
- Interrupting a tool with Ctrl+C will correctly wait for the tool to exit (#513)

# Version 0.6.0

- Allow installing 3rd-party binaries from private registries (#469)

# Version 0.5.7

- Prevent corrupting local cache by downloading tools to temp directory (#498)

# Version 0.5.6

- Improve expected behavior with Yarn in projects (#470)
- Suppress an erroneous "toolchain" key warning message (#486)

# Version 0.5.5

- Proper support for relative paths in Bin hooks (#468)
- Diagnostic messages for shims with `VOLTA_LOGLEVEL=debug` (#466)
- Preserve user order for multiple tool installs (#479)

# Version 0.5.4

- Show additional diagnostic messages when run with `--verbose` (#455)

# Version 0.5.3

- Prevent unnecessary warning output when not running interactively (#451)
- Fix a bug in load script for fish shell on Linux (#456)
- Improve wrapping behavior for warning messages (#453)

# Version 0.5.2

- Improve error messages when running a project-local binary fails (#426)
- Fix execution of user binaries on Windows (#445)

# Version 0.5.1

- Add per-project hooks configuration in `<PROJECT_ROOT>/.volta/hooks.json` (#411)
- Support backwards compatibility with `toolchain` key in `package.json` (#434)

# Version 0.5.0

- Rename to Volta: The JavaScript Launcher ⚡️
- Change `package.json` key to `volta` from `toolchain` (#413)
- Update `volta completions` behavior to be more usable (#416)
- Improve `volta which` to correctly find user tools (#419)
- Remove unneeded lookups of `package.json` files (#420)
- Cleanup of error messages and extraneous output (#421, #422)

# Version 0.4.1

- Allow tool executions to pass through to the system if no Notion platform exists (#372)
- Improve installer support for varied Linux distros

# Version 0.4.0

- Update `notion install` to use `tool@version` formatting for specifying a tool (#383, #403)
- Further error message improvements (#344, #395, #399, #400)
- Clean up bugs around installing and running packages (#368, #390, #394, #396)
- Include success messages when running `notion install` and `notion pin` (#397)

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
