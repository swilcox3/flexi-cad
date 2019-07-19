import * as ops from '../operations/operations'
var myController = require('./controller')
var mySingleton = new myController().getInstance();
var Mousetrap = require('mousetrap')

Mousetrap.bind('ctrl+z', () => ops.undoLatest())
Mousetrap.bind('command+z', () => ops.undoLatest())
Mousetrap.bind('ctrl+y', () => ops.redoLatest())
Mousetrap.bind('command+y', () => ops.redoLatest())
Mousetrap.bind('del', () => mySingleton.deleteSelected())
Mousetrap.bind('esc', () => mySingleton.cancel())
Mousetrap.bind('escape', () => mySingleton.cancel())
Mousetrap.bind('ctrl', () => mySingleton.ctrlDown(), 'keydown')
Mousetrap.bind('ctrl', () => mySingleton.ctrlUp(), 'keyup')
Mousetrap.bind('mod+c', () => mySingleton.setClipboard())
Mousetrap.bind('mod+v', () => mySingleton.pasteClipboard())