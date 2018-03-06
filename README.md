# Notion: the hassle-free node.js manager
[![Build Status](https://travis-ci.org/notion-cli/notion.svg?branch=master)](https://travis-ci.org/notion-cli/notion)
[![Build status](https://ci.appveyor.com/api/projects/status/2cohtlutserh8jfb/branch/master?svg=true)](https://ci.appveyor.com/project/stefanpenner/notion/branch/master)

**This project is just getting started.**

## Unix installation

First-time setup (this will get automated more):
```sh
$ cargo build --release
$ cd support/unix
$ ./build.sh ../../target/release/notion ../../target/release/node ../../target/release/launchbin ../../target/release/launchscript
$ ./install.sh
```
The first time you install, you'll need to open a new terminal to start using Notion.

To reinstall an updated build, first remove everything from `~/.notion` except for the configuration file (again, this will get automated!):
```sh
$ rm -rf ~/.notion/bin ~/.notion/cache ~/.notion/state.toml ~/.notion/launch* ~/.notion/shim ~/.notion/versions
```
and then follow the setup steps above.

## Windows installation

Working on it!

## Demo

There's a sample `package.json` in this repo so just cd into the repo and run:

```sh
$ node -e 'console.log(`this project uses node v${process.version}`)'
Installing v6.11.3 [=============>           ]  50%
this project uses node v6.11.3
$ node -e 'console.log(`this project uses node v${process.version}`)'
this project uses node v6.11.3
$ notion uninstall 6.11.3
$ node -e 'console.log(`this project uses node v${process.version}`)'
Installing v6.11.3 [==================>      ]  76%
this project uses node v6.11.3
```

# Development

## Requirements

Notion is intended to compile with all versions of Rust newer than 1.20.

## License

Notion is licensed under a [BSD 2-clause license](https://github.com/notion-cli/notion/blob/master/LICENSE).

## Code of Conduct

Contribution to Notion is organized under the terms of the [Contributor Covenant Code of Conduct](https://github.com/notion-cli/notion/blob/master/CODE_OF_CONDUCT.md).
The maintainer of Notion, Dave Herman, personally promises to work actively to uphold that code of conduct.
We aim to foster a community that is welcoming, inclusive, empathetic, and kind.
If you share those goals and want to have a ton of building cool JavaScript tools and playing with Rust, we invite you to join us!
