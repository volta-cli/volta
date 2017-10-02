# Nemo

Here's the setup (currentl manual until we implement installation scripts) for the initial demo:

```
$ cargo build --release
$ mkdir -p ~/.nemo/bin
$ ln -s $(pwd)/target/release/proxy ~/.nemo/bin/node
$ ln -s $(pwd)/target/release/proxy ~/.nemo/bin/npm
$ ln -s $(pwd)/target/release/nemo ~/.nemo/bin/nemo
$ export PATH="~/.nemo/bin:$PATH"
```

Then here's the demo:

```
$ node -e 'console.log(`this project uses node v${process.version}`)'
Installing v6.11.3 [=============>           ]  50%
this project uses node v6.11.3
$ node -e 'console.log(`this project uses node v${process.version}`)'
this project uses node v6.11.3
$ nemo uninstall 6.11.3
$ node -e 'console.log(`this project uses node v${process.version}`)'
Installing v6.11.3 [==================>      ]  76%
this project uses node v6.11.3
```

