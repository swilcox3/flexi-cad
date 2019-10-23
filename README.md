# flexi-cad
The goal of this project is to design an architectural CAD product that uses largely the same code for three different versions: workstation, thin-client/thick-server, and browser based.  The name is dumb, but it works for now.

Follow the setup instructions for Neon: https://neon-bindings.com/docs/getting-started
Then install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/
Clone this repository.
For workstation configuration, go to ui/electron and npm install, then npm run start.  A dialog will pop up saying "Connect to server?"  Choose no.
For thin-client configuration, first go to /server and cargo run --release.  Then execute the above instructions to start the electron client and choose yes on the dialog.
For browser, start the server as above, then go to /ui/browser and npm install, then npm run start.  A browser tab will pop up.
All three configurations build and successfully get into the application now.
