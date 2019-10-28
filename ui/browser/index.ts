import * as ops from '../src/operations/operations';
//@ts-ignore
import WebsocketAsPromised from "websocket-as-promised";

declare global {
  export type JsDimension = import("../data-model-wasm/pkg/index").JsDimension;
  export type JsWall = import("../data-model-wasm/pkg/index").JsWall;
  export type JsDoor = import("../data-model-wasm/pkg/index").JsDoor;
  export type JsSlab = import("../data-model-wasm/pkg/index").JsSlab;
}
window.addEventListener('DOMContentLoaded', () => {
  import("../data-model-wasm/pkg/index").then(mod => {
    var connection = "ws://" + window.location.host + "/ws";
    console.log(connection);
    console.log("hi there!");
    var user = ops.initialize(mod);
    connection = connection + "?user_id=" + user;
    var conn = new WebsocketAsPromised(connection, {
      packMessage: (data: any) => JSON.stringify(data),
      unpackMessage: (data: any) => JSON.parse(data),
    });
    ops.setConnection(conn).then(() => {
      console.log("prompting: ");
      var file = prompt("File name:", "defaultNew.flx");
      console.log(file);
      ops.initFile(document.getElementById('renderCanvas') as HTMLCanvasElement, file);
    });
  });
});
window.addEventListener("unload", () => {
  ops.closeFile()
})
