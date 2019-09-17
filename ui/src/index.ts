var ops = require('./operations/operations')

if(navigator.userAgent.toLowerCase().indexOf(' electron/') == -1) {
    var connection = "ws://127.0.0.1:80/ws";
    ops.setConnection(connection);
    ops.initFile(document.getElementById('render-canvas'));
}
else {
    type IpcRendererEvent = import("electron").IpcRendererEvent;
    var {ipcRenderer} = require('electron');
    ipcRenderer.on('newFile', function(event: IpcRendererEvent, connection: string) {
        if(connection !== undefined) {
            ops.setConnection(connection)
        }
        ops.initFile(document.getElementById('render-canvas'))
    })
    ipcRenderer.on('openFile', function(event: IpcRendererEvent, path: string, connection: string) {
        if(connection !== undefined) {
            ops.setConnection(connection)
        }
        ops.openFile(path, document.getElementById('render-canvas'))
    });
    ipcRenderer.on('saveFile', function(event: IpcRendererEvent) {
        ops.saveFile()
    });
    ipcRenderer.on('saveAsFile', function(event: IpcRendererEvent, path: string) {
        ops.saveAsFile(path)
    });
}

var keys = require('./ui/key_events')