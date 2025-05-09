# Rencounter counter 0.0.2

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

### Linux
- open terminal
- go to the directory of app
- run the following command
```bash
./rencounter_counter
```
### Mac
- if your mac is from before 2020 (intel chip), choose macos-latest-x64.zip, if your mac is from after 2020 (m1/m2/3 chip) choose macos-latest-arm64.zip
- double click on the app
- hit s 
- mac will ask you to give terminal permissions to take screenshots
- hit ok
- close terminal and run app again

## Download stand alone app
If you don't want to install Rust and run the app from the terminal, you can download the stand alone app from the following link
[Download the app(https://github.com/rengare/rencounter_counter/releases)

![image](https://github.com/rengare/rencounter_counter/assets/10849982/d9715798-f952-43ef-9e88-2ee555a84ddb)



### Recomendation
Rencounter grabs a screenshot of your screen, cuts the height in half and take upper half (see picture)
It is recommended to play the game with 1920x1080 resolution and make the game bigger than half the width. Though you can still play with lower resolution but you need to find a good spot.
![image](https://github.com/rengare/rencounter_counter/assets/10849982/a32e8c46-824c-4a8f-ae48-856cf479b6e8)

## How build it from source 

### Linux
1. Install dependencies
Ubuntu / Mint / Debian / PopOS
```bash
sudo apt-get install build-essential libxcb-shm0-dev libxcb-randr0-dev xcb git libxcb1 libxrandr2 libdbus-1-3
```

### Windows
1. Install Visual Studio 2022 with C++ build tools https://visualstudio.microsoft.com/downloads/

### Mac
1. Install Xcode from the App Store

### All platforms
1. Clone the repository
2. Install Rust language from [here](https://www.rust-lang.org/tools/install) 
3. Run the following command in the terminal
```bash
git clone github.com/rengare/rencounter_counter
cd rencounter_counter
cargo run --release
```

## TODO
- [x] Add a stand alone app
- [x] Test on Windows and Mac(pre M1 and post)
- [x] Show top 5 mons with the most encounters
- [x] Remove reading/writing from file/to file every 100MS, use buffer instead
- [x] Save number of encounter per mon
- [ ] Retrain AI model with Pokemmo fonts or use different fonts that work better with the current model
- [x] Detect when the game is covered by another app and pause the counter

