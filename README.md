# Notion: the hassle-free node.js manager

**This project is just getting started.**

Here's the setup (currently manual until we implement installation scripts) for the initial demo:

```
$ cargo build --release
$ mkdir -p ~/.notion/bin
$ ln -s $(pwd)/target/release/proxy ~/.notion/bin/node
$ ln -s $(pwd)/target/release/proxy ~/.notion/bin/npm
$ ln -s $(pwd)/target/release/notion ~/.notion/bin/notion
$ export PATH="~/.notion/bin:$PATH"
```

Then here's the demo:

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
