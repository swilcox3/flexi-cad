import * as ops from '../operations/operations'
var myController = require('./controller')
var mySingleton = new myController().getInstance();
var Mousetrap = require('mousetrap')

Mousetrap.bind('ctrl+z', () => ops.undoLatest())
Mousetrap.bind('command+z', () => ops.undoLatest())
Mousetrap.bind('ctrl+y', () => ops.redoLatest())
Mousetrap.bind('command+y', () => ops.redoLatest())
Mousetrap.bind('del', () => mySingleton.deleteSelected())