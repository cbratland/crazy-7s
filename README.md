# crazy 7s
A crazy 8s/uno-like online card game. Built with [Bevy](https://bevyengine.org/) and [matchbox](https://github.com/johanhelsing/matchbox).

## Building
The project can be built with
```sh
cargo build --release
```
or run with
```sh
cargo run --release
```
Note that this could take a while to build the first time, as it needs to download and compile all of the dependencies.

Additionally, if you want to browse the documentation for the project, you can run
```sh
cargo doc --document-private-items --open
```
and it will open a web page with the modules and functions, and their descriptions, listed.

## Running
The project requires a server that it can connect to in order to enable peer-to-peer communication. The default [matchbox](https://github.com/johanhelsing/matchbox) server can be used by running
```sh
cargo install matchbox_server
matchbox_server
```
Then, you can run multiple instances of the game and host on one and join on the others by typing in the code displayed on the host's screen.

### Disclaimer
This game was built for fun, to be played with friends. The game networking is not very secure, and people can easily cheat by looking at the network traffic.

### TODO
There are a few things that I'd like to add to the game, but haven't gotten around to yet. These are listed below:
- [ ] better wild menu appearance
- [ ] don't show wild menu when last card is a wild
- [ ] some indication of turn direction
- [ ] better turn indicator
- [ ] in game menu
- [ ] player list in lobby
- [ ] handle game end (if host leaves)
- [ ] handle disconnects (remove player from game)
- [ ] handle server connection failure
- [ ] join should check if game is in progress (may need to modify server)
- [ ] player colors or avatars (?)
