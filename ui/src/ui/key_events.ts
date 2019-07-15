import * as ops from '../operations/operations'
var Mousetrap = require('mousetrap')

Mousetrap.bind('ctrl+z', () => ops.undoLatest())
Mousetrap.bind('command+z', () => ops.undoLatest())
Mousetrap.bind('ctrl+y', () => ops.redoLatest())
Mousetrap.bind('command+y', () => ops.redoLatest())