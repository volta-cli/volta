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
