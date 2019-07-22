import * as ops from '../operations/operations'
var myController = require('./controller')
var mySingleton = new myController().getInstance();
var Mousetrap = require('mousetrap')

Mousetrap.bind('mod+z', () => ops.undoLatest())
Mousetrap.bind('mod+y', () => ops.redoLatest())
Mousetrap.bind('del', () => mySingleton.onDeleteKey())
Mousetrap.bind('esc', () => mySingleton.cancel())
Mousetrap.bind('escape', () => mySingleton.cancel())
Mousetrap.bind('mod', () => mySingleton.ctrlDown(), 'keydown')
Mousetrap.bind('mod', () => mySingleton.ctrlUp(), 'keyup')
Mousetrap.bind('mod+c', () => mySingleton.setClipboard())
Mousetrap.bind('mod+v', () => mySingleton.pasteClipboard())