# Demo blockers

- windows installer isn't killing the node installation directories
- unix script launchers
- unix bash install script
- pedagogy/PR: how should we talk about this approach?
  - terminology for the basic technique (as well as the alternatives)
  - what is the high-level intuition without diving deep into how it all works?

# Basic functionality

- add npx to the set of {bin,script}stubs
- package executables
- proper behavior for executable-not-found
- acceptance test harness
- complete version parsing (e.g. "8" because "8.latest" and "8.5" becomes "8.5.latest")

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
- `notion selfupdate`
- hooks for corporate metrics
- offline support
