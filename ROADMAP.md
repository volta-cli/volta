# Demo blockers

- `nodeup current --local` shouldn't be calling `config::read()`
- user-global state (`nodeup current --global` et al)
- unix script launchers
- pedagogy/PR: how should we talk about this approach?
  - terminology for the basic technique (as well as the alternatives)
  - what is the high-level intuition without diving deep into how it all works?
- windows installer isn't killing the node installation directories
- unix bash install script

# Basic functionality

- add npx to the set of {bin,script}stubs
- proper behavior for executable-not-found

# Quality improvements

- add UI to windows installer
- appveyor tests
- appveyor deploy script:
  - generate msi with wix toolset
    - `candle -ext WixUtilExtension support\windows\Nodeup.wxs`
    - `light -ext WixUtilExtension Nodeup.wixobj`
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
