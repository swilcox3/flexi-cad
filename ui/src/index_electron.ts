var keys = require('./app/ui/key_events')
var {ipcRenderer} = require('electron');

var ops = require('./app/operations/operations')
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
