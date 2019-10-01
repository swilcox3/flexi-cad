import * as ops from '../src/operations/operations';
//@ts-ignore
import WebsocketAsPromised from "websocket-as-promised";
import '../src/ui/key_events';

declare global {
    export type JsDimension = import("../data-model-wasm/pkg/index").JsDimension;
    export type JsWall = import("../data-model-wasm/pkg/index").JsWall;
    export type JsDoor = import("../data-model-wasm/pkg/index").JsDoor;
}
window.addEventListener('DOMContentLoaded', () => {
  console.log("made it index")
  import("../data-model-wasm/pkg/index").then( mod => {
    console.log("loaded")
    console.log("After ops")
    var connection = "ws://127.0.0.1:80/ws";
    var user = ops.initialize(mod);
    connection = connection + "?user_id=" + user;
    var conn = new WebsocketAsPromised(connection, {
      packMessage: (data:any) => JSON.stringify(data),
      unpackMessage: (data:any) => JSON.parse(data),
    });
    ops.setConnection(conn).then(() => {
        console.log("connection set")
        ops.initFile(document.getElementById('renderCanvas') as HTMLCanvasElement);
        console.log("after init file")
    });
  });
});
