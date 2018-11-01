# Notion: the hassle-free node.js manager
[![Build Status](https://travis-ci.org/notion-cli/notion.svg?branch=master)](https://travis-ci.org/notion-cli/notion)
[![Build status](https://ci.appveyor.com/api/projects/status/2cohtlutserh8jfb/branch/master?svg=true)](https://ci.appveyor.com/project/stefanpenner/notion/branch/master)

**This project is just getting started.**

## Unix installation

On macOS or Linux, you can install Notion with this command:

```sh
$ curl -sSLf https://get.notionjs.com | bash
```

## Windows installation

One of Notion's primary goals is to have first-class Windows support out of the box. Several contributors to Notion use Windows as our primary development machines, and this is very important to us.

We haven't written real installers for any platform yet, and our current status on Windows is that we've learned how to create Windows installers and solved the major design questions that we've identified. If you'd like to help with a Windows installer, please [get in touch](#community)!

## Source installation

### Unix

First-time setup (this will get automated more):
```sh
$ cargo build --release
$ ./dev/unix/build.sh
$ ./dev/unix/install.sh
```
The first time you install, you'll need to open a new terminal to start using Notion.

To reinstall an updated build, first remove everything from `~/.notion` except for the configuration file (again, this will get automated!):
```sh
$ rm -rf ~/.notion/bin ~/.notion/cache ~/.notion/state.toml ~/.notion/launch* ~/.notion/versions
```
and then follow the setup steps above.

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

## Community

We use [Discord](https://discordapp.com/) for community discussion. You can use this [permanent invite](https://discord.gg/hgPTz9A) to join our Discord server!

## Requirements

Notion is intended to compile with all versions of Rust newer than 1.20.

## Tests

To run the tests in this repo, run the following:

```
cargo test --all --features mock-network
```

When running tests in CI we also check that code formatting matches the
`rustfmt` generated style, follow the steps in the Formatting section below to
avoid issue.

## Formatting

To ensure your code is formatted correctly, run the following:

```
./dev/unix/rustfmt.sh
```

## License

Notion is licensed under a [BSD 2-clause license](https://github.com/notion-cli/notion/blob/master/LICENSE).

## Code of Conduct

Contribution to Notion is organized under the terms of the [Contributor Covenant Code of Conduct](https://github.com/notion-cli/notion/blob/master/CODE_OF_CONDUCT.md).
The maintainer of Notion, Dave Herman, personally promises to work actively to uphold that code of conduct.
We aim to foster a community that is welcoming, inclusive, empathetic, and kind.
If you share those goals and want to have a ton of building cool JavaScript tools and playing with Rust, we invite you to join us!
