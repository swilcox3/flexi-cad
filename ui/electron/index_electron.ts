import('./app/ui/key_events')
import {ipcRenderer} from 'electron';

declare global {
    export type JsDimension = import("./native/index").JsDimension;
    export type JsWall = import("./native/index").JsWall;
    export type JsDoor = import("./native/index").JsDoor;
}
import * as mod from "./native/index.node";
var ops = require('./app/operations/operations')
ops.initialize(mod);
ipcRenderer.on('newFile', function(event: string, connection: string) {
    if(connection !== undefined) {
        ops.setConnection(connection).then( () => {
            ops.initFile(document.getElementById('render-canvas'))
        })
    }
    else {
        ops.initFile(document.getElementById('render-canvas'))
    }
})
ipcRenderer.on('openFile', function(event: string, path: string, connection: string) {
    if(connection !== undefined) {
        ops.setConnection(connection).then( () => {
            ops.openFile(path, document.getElementById('render-canvas'))
        })
    }
    else {
        ops.openFile(path, document.getElementById('render-canvas'))
    }
});
ipcRenderer.on('saveFile', function(event: string) {
    ops.saveFile()
});
ipcRenderer.on('saveAsFile', function(event: string, path: string) {
    ops.saveAsFile(path)
});
