# Rencounter counter

## Description
Encounter counter for the Pokemmo. 
It is a simple tool to keep track of the number of encounters in the game.

## Features
- Automaticaly Count the number of encounters
- Reset the counter
- Automaticaly state of the counter to a file
- Automaticaly load the state of the counter from a file if exists
- Start / pause mechanism


## How to use
[![Watch the video](https://img.youtube.com/vi/zjVu3N2xFzA/0.jpg)](https://www.youtube.com/watch?v=zjVu3N2xFzA)

## How to install

(stand alone app available soon)

1. Clone the repository
2. Install Rust language from [here](https://www.rust-lang.org/tools/install) 
3. Run the following command in the terminal
```bash
git clone github.com/rengare/rencounter-counter
cd rencounter-counter
cargo run --release
```

## TODO
- [ ] Add a stand alone app
- [ ] Save number of encounter per mon
- [ ] Show top 5 mons with the most encounters
- [ ] Retrain AI model with Pokemmo fonts or use different fonts that work better with the current model
- [ ] Test on Windows and Mac(pre M1 and post)
- [ ] Detect when the game is covered by another app and pause the counter

