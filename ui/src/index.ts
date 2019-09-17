var ops = require('./operations/operations')

// #if navigator.userAgent.toLowerCase().indexOf(' electron/') == -1
var connection = "ws://127.0.0.1:80/ws";
ops.setConnection(connection);
ops.initFile(document.getElementById('render-canvas'));
// #else
var {ipcRenderer} = require('electron');
ipcRenderer.on('newFile', function(event: string, connection: string) {
    if(connection !== undefined) {
        ops.setConnection(connection)
    }
    ops.initFile(document.getElementById('render-canvas'))
})
ipcRenderer.on('openFile', function(event: string, path: string, connection: string) {
    if(connection !== undefined) {
        ops.setConnection(connection)
    }
    ops.openFile(path, document.getElementById('render-canvas'))
});
ipcRenderer.on('saveFile', function(event: string) {
    ops.saveFile()
});
ipcRenderer.on('saveAsFile', function(event: string, path: string) {
    ops.saveAsFile(path)
});
// #endif

var keys = require('./ui/key_events')