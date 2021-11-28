# ethminer-gui: Prototype gui app for mining ethereum
![Alt text](icon.png)

Simple portable gui app which acts as a wrapper around the ethminer cli.

Built with rust and [egui](https://github.com/emilk/egui).

## Features
Easy configuration of select miner and pool settings

View console output of miner with an in-app preview

Fast: window updates use lazy loading for low cpu usage

Multiplatform support for Windows and linux

## How to build and run
You will need to install cargo using rustup: https://www.rust-lang.org/tools/install

From the root directory of this repo, run ```cargo run --release```

The compiled binary can be found in ```target/release/ethminer-gui```

## Future Improvements
Allow specification of multiple mining pools

Add more device settings, for example temperature limits and usage limits

Add deserialization and serialization using ```serde``` for saving mining profile state on shutdown

Add auto start setting for mining on app launch

Add parsing of hashrate from the stdout, in order to create an egui plot with historical hashrate. 

Make the app look prettier