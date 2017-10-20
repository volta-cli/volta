# Demo blockers

- UI to set global state: `nodeup use --global 8.6.0`
- bug: windows path length bugs when unpacking node 8.5
- windows installer isn't killing the node installation directories
- unix script launchers
- unix bash install script
- pedagogy/PR: how should we talk about this approach?
  - terminology for the basic technique (as well as the alternatives)
  - what is the high-level intuition without diving deep into how it all works?

# Basic functionality

- add npx to the set of {bin,script}stubs
- proper behavior for executable-not-found
- acceptance test harness

# Quality improvements

- add UI to windows installer
- appveyor tests
- appveyor deploy script:
  - generate msi
  - publish to GitHub release
- travis tests
- travis deploy script
- windows UX:
  - try to get zip-rs to land https://github.com/mvdnes/zip-rs/pull/37
  - or maybe just show a spinner while downloading the zip
- diagnostics (look for other node installs that could be conflicting)
- proper grammar for node version specifiers
- apt, homebrew, chocolatey releases
- `nodeup selfupdate`
- hooks for corporate metrics
- offline support
