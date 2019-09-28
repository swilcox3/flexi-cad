import './app/ui/key_events'
import {ipcRenderer} from 'electron';
import * as ops from '../src/operations/operations';

declare global {
    export type JsDimension = import("./native/index").JsDimension;
    export type JsWall = import("./native/index").JsWall;
    export type JsDoor = import("./native/index").JsDoor;
}
console.log("made it")
var mod = require("./native/index.node");
ops.initialize(mod);
ipcRenderer.on('newFile', function(event: string, connection: string) {
    console.log(connection)
    if(connection !== undefined) {
        ops.setConnection(connection).then( () => {
            ops.initFile(document.getElementById('render-canvas') as HTMLCanvasElement)
        })
    }
    else {
        ops.initFile(document.getElementById('render-canvas') as HTMLCanvasElement)
    }
})
ipcRenderer.on('openFile', function(event: string, path: string, connection: string) {
    if(connection !== undefined) {
        ops.setConnection(connection).then( () => {
            ops.openFile(path, document.getElementById('render-canvas') as HTMLCanvasElement)
        })
    }
    else {
        ops.openFile(path, document.getElementById('render-canvas') as HTMLCanvasElement)
    }
});
ipcRenderer.on('saveFile', function(event: string) {
    ops.saveFile()
});
ipcRenderer.on('saveAsFile', function(event: string, path: string) {
    ops.saveAsFile(path)
});
