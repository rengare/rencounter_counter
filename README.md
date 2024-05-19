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
./rencounter_counter_linux
```
### Mac
- double click on the app
- hit s 
- mac will ask you to give terminal permissions to take screenshots
- hit ok
- close terminal and run app again

## Download stand alone app
If you don't want to install Rust and run the app from the terminal, you can download the stand alone app from the following link
![Download the app](https://github.com/rengare/rencounter_counter/actions)

Click on latest build, then click on the artifact and download the app for your platform
![image](https://github.com/rengare/rencounter_counter/assets/10849982/1372904e-e224-4caa-93a3-9f4d889ae886)
![image](https://github.com/rengare/rencounter_counter/assets/10849982/1ea1fbfb-46af-4af9-8c8f-5ee07bb60c69)


### Recomendation
Rencounter grabs a screenshot of your screen, cuts the height in half and take upper half (see picture)
It is recommended to play the game with 1920x1080 resolution and make the game bigger than half the width. Though you can still play with lower resolution but you need to find a good spot.
![image](https://github.com/rengare/rencounter_counter/assets/10849982/a32e8c46-824c-4a8f-ae48-856cf479b6e8)

## How build it from source 

### Linux
1. Install dependencies
Ubuntu / Mint / Debian / PopOS
```bash
sudo apt-get install build-essential libxcb-shm0-dev libxcb-randr0-dev xcb git
```
Fedora / RedHat
```bash
sudo dnf install @development-tools gcc gcc-c++ make git libxcb-devel xcb-util-keysyms-devel xcb-util-devel xcb-util-wm-devel
```
Arch 
```bash
sudo pacman -S base-devel git xcb-util xcb-util-keysyms xcb-util-wm
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
- [ ] Save number of encounter per mon
- [ ] Retrain AI model with Pokemmo fonts or use different fonts that work better with the current model
- [ ] Detect when the game is covered by another app and pause the counter
- [ ] Add a way for user to select encounter area on screen

