var ops = require('./operations/operations')
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

var keys = require('./ui/key_events')