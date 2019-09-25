import * as ops from './operations/operations';
import './ui/key_events'

declare global {
    export type JsDimension = import("../data-model-wasm/pkg/index").JsDimension;
    export type JsWall = import("../data-model-wasm/pkg/index").JsWall;
    export type JsDoor = import("../data-model-wasm/pkg/index").JsDoor;
}
window.addEventListener('DOMContentLoaded', () => {
  console.log("made it index")
  import("../data-model-wasm/pkg/index").then( mod => {
    console.log("loaded")
    ops.initialize(mod);
    console.log("After ops")
    var connection = "ws://127.0.0.1:80/ws";
    ops.setConnection(connection).then( () => {
        console.log("connection set")
        ops.initFile(document.getElementById('renderCanvas') as HTMLCanvasElement);
        console.log("after init file")
    });
  });
});
