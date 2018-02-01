# Notion: the hassle-free node.js manager
[![Build Status](https://travis-ci.org/notion-cli/notion.svg?branch=master)](https://travis-ci.org/notion-cli/notion)

**This project is just getting started.**

## Unix installation

First-time setup (this will get automated more):
```
$ cargo build --release
$ cd support/unix
$ ./build.sh ../../target/release/notion ../../target/release/launchbin ../../target/release/launchscript
$ ./install.sh
```
The first time you install, you'll need to open a new terminal to start using Notion.

To reinstall an updated build, first remove everything from `~/.notion` except for the configuration file (again, this will get automated!):
```
$ rm -rf ~/.notion/bin ~/.notion/cache ~/.notion/state.toml ~/.notion/launch* ~/.notion/shim ~/.notion/versions
```
and then follow the setup steps above.

## Windows installation

Working on it!

## Demo

There's a sample `package.json` in this repo so just cd into the repo and run:

```
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
