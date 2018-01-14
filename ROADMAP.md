# Demo blockers

- unix script launchers
- unix bash install script
- pedagogy/PR: work out how to explain the shims approach

# Basic functionality

- windows installer isn't killing the node installation directories
- add npx to the set of shims
- package executables
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
- proper grammar for node version specifiers
- apt, homebrew, chocolatey releases
- `notion selfupdate`
- hooks for corporate metrics
- offline support
- installer should check for existing global packages and install them
- `npm install -g` should be intercepted and install global package shims
