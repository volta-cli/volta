# Demo blockers

- unix script launchers
- windows installer isn't killing the node installation directories
- figure out how we want to combine io::Error and reqwest::Error and ZipError
  - just use io::Error and wrap reqwest errors with io::Error::new()?
  - or look into error-chain?
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
