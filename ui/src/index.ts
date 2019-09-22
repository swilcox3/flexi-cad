var keys = require('./ui/key_events')

export type DataModelMod = typeof import("../data-model-wasm/pkg/index");

declare global {
    export type JsDimension = import("../data-model-wasm/pkg/index").JsDimension;
    export type JsWall = import("../data-model-wasm/pkg/index").JsWall;
    export type JsDoor = import("../data-model-wasm/pkg/index").JsDoor;
}
console.log("made it index")
var loaded = import("../data-model-wasm/pkg/index").then( mod => {
    console.log("loaded")
    var ops = require('./operations/operations')(mod);
    console.log("After ops")
    var connection = "ws://127.0.0.1:80/ws";
    ops.setConnection(connection).then( () => {
        console.log("connection set")
        ops.initFile(document.getElementById('render-canvas'));
        console.log("after init file")
    });
});