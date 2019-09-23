import './ui/key_events'

export type DataModelMod = typeof import("../data-model-wasm/pkg/index");

declare global {
    export type JsDimension = import("../data-model-wasm/pkg/index").JsDimension;
    export type JsWall = import("../data-model-wasm/pkg/index").JsWall;
    export type JsDoor = import("../data-model-wasm/pkg/index").JsDoor;
}
console.log("made it index")
var loaded = import("../data-model-wasm/pkg/index").then( mod => {
    console.log("loaded")
    import('./operations/operations').then(ops => {
        ops.initialize(mod);
        console.log("After ops")
        var connection = "ws://127.0.0.1:80/ws";
        ops.setConnection(connection).then( () => {
            console.log("connection set")
            ops.initFile(document.getElementById('render-canvas') as HTMLCanvasElement);
            console.log("after init file")
        });
    })
});