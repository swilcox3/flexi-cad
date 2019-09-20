# flexi-cad
The goal of this project is to design an architectural CAD product that uses largely the same code for three different versions: workstation, thin-client/thick-server, and browser based.  The name is dumb, but it works for now.

Follow the setup instructions for Neon: https://neon-bindings.com/docs/getting-started
Clone this repository, then navigate to /ui in a terminal and execute:
npm install
Then go to /ui/data-model-wasm and npm install there as well.  Then back up to ui and run:
npm run start

Currently you'll be presented with a dialog on startup saying "Connect to server?".  Choose no, the server isn't started, so you'll start in workstation mode.  If you want to run in server/client configuration, go to /server and run cargo run --release, then go back to /ui and run npm run start and choose yes at the dialog.  You can start two instances of the client with two terminals, one calling npm run start, then once that's done running npm run start_nb (which skips the build step) in the second terminal.   
