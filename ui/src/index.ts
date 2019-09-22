var keys = require('./ui/key_events')

export type DataModelMod = typeof import("../data-model-wasm/pkg/index");

declare global {
    export type JsDimension = import("../data-model-wasm/pkg/index").JsDimension;
    export type JsWall = import("../data-model-wasm/pkg/index").JsWall;
    export type JsDoor = import("../data-model-wasm/pkg/index").JsDoor;
}
if(navigator.userAgent.toLowerCase().indexOf(' electron/') == -1) {
    var loaded = import("../data-model-wasm/pkg/index").then( mod => {
        var ops = require('./operations/operations')(mod);
        var connection = "ws://127.0.0.1:80/ws";
        ops.setConnection(connection).then( () => {
            ops.initFile(document.getElementById('render-canvas'));
        });
    });
}