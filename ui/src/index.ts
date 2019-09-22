var ops = require('./operations/operations')
var keys = require('./ui/key_events')
if(navigator.userAgent.toLowerCase().indexOf(' electron/') == -1) {
    var connection = "ws://127.0.0.1:80/ws";
    ops.setConnection(connection).then( () => {
        ops.initFile(document.getElementById('render-canvas'));
    });
}