# nodeup

Here's the setup (currently manual until we implement installation scripts) for the initial demo:

```
$ cargo build --release
$ mkdir -p ~/.nodeup/bin
$ ln -s $(pwd)/target/release/proxy ~/.nodeup/bin/node
$ ln -s $(pwd)/target/release/proxy ~/.nodeup/bin/npm
$ ln -s $(pwd)/target/release/nodeup ~/.nodeup/bin/nodeup
$ export PATH="~/.nodeup/bin:$PATH"
```

Then here's the demo:

```
$ node -e 'console.log(`this project uses node v${process.version}`)'
Installing v6.11.3 [=============>           ]  50%
this project uses node v6.11.3
$ node -e 'console.log(`this project uses node v${process.version}`)'
this project uses node v6.11.3
$ nodeup uninstall 6.11.3
$ node -e 'console.log(`this project uses node v${process.version}`)'
Installing v6.11.3 [==================>      ]  76%
this project uses node v6.11.3
```

