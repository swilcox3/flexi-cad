const gui = require('./gui')
import * as math from '../utils/math'
import { openSync } from 'fs';
import * as ops from '../operations/operations'

interface Tool
{
    onMouseDown(pt: math.Point3d, hovered?:BABYLON.Mesh):undefined,
    onMouseMove(pt: math.Point3d, hovered?:BABYLON.Mesh):boolean,
    cancel():undefined,
    finish(pt: math.Point3d, picked?:BABYLON.Mesh):undefined
}

class SelectionController
{
    private selectedObjs: Set<BABYLON.Mesh>;
    constructor() {
        this.selectedObjs = new Set();
    }

    getSelectedObjs()
    {
        return this.selectedObjs
    }

    deselectAll()
    {
        this.selectedObjs.forEach((obj) => {
            var mat = obj.material as BABYLON.StandardMaterial;
            mat.diffuseColor = BABYLON.Color3.Gray();
        })
        this.selectedObjs.clear();
    }

    addObject(mesh: BABYLON.Mesh)
    {
        var mat = mesh.material as BABYLON.StandardMaterial;
        mat.diffuseColor = BABYLON.Color3.Green();
        this.selectedObjs.add(mesh)
    }

    selectObject(mesh: BABYLON.Mesh)
    {
        this.deselectAll();
        this.addObject(mesh);
    }
}

class UIController
{
    private activeTool: Tool
    private selection: SelectionController
    private ctrlPressed: boolean;
    private moveEvent: string;
    constructor() {
        this.activeTool = null;
        this.selection = new SelectionController();
        this.ctrlPressed = false;
    }

    setActiveTool(tool:Tool)
    {
        if(this.activeTool != null)
        {
            this.activeTool.cancel()
        }
        this.activeTool = tool
    }

    leftDown(mesh: BABYLON.Mesh)
    {
        if(!this.ctrlPressed) {
            this.selection.selectObject(mesh)
        }
        else {
            this.selection.addObject(mesh)
        }
    }

    leftClick(pt:math.Point3d, mesh: BABYLON.Mesh)
    {
        if(this.activeTool != null)
        {
            this.activeTool.onMouseDown(pt, mesh)
        }
        else if(mesh != null)
        {
            if(!this.ctrlPressed) {
                this.selection.selectObject(mesh)
            }
            else {
                this.selection.addObject(mesh)
            }
            gui.guiInstance.setObjectOverlay(this.selection.getSelectedObjs())
        }
        else if(mesh == null)
        {
            this.selection.deselectAll();
            gui.guiInstance.clearObjectOverlay();
        }
    }

    rightClick(pt:math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.activeTool != null)
        {
            this.activeTool.finish(pt, picked)
            this.activeTool = null
        }
        else if (picked == null)
        {
            this.selection.deselectAll();
        }
    }

    mouseMove(pt:math.Point3d, hovered: BABYLON.Mesh)
    {
        if(this.activeTool != null)
        {
            return this.activeTool.onMouseMove(pt, hovered)
        }
        return true;
    }

    ctrlDown()
    {
        this.ctrlPressed = true;
    }

    ctrlUp()
    {
        this.ctrlPressed = false;
    }

    deleteSelected()
    {
        var event = ops.beginUndoEvent("Delete objs");
        this.selection.getSelectedObjs().forEach((obj) => {
            ops.deleteObject(event, obj.name)
        });
        ops.endUndoEvent(event);
        this.selection.deselectAll()
    }

    moveSelected(ev: any)
    {
        if(!this.moveEvent) {
            this.moveEvent = ops.beginUndoEvent("Move objects")
            this.selection.getSelectedObjs().forEach((mesh) => {
                ops.takeUndoSnapshot(this.moveEvent, mesh.name)
            })
            ops.suspendEvent(this.moveEvent)
        }
        var modelDelta = math.transformGraphicToModelCoords(ev.delta) 
        let names: Array<string> = []
        this.selection.getSelectedObjs().forEach((mesh) => {
            names.push(mesh.name)
        })
        ops.moveObjs(this.moveEvent, names, modelDelta)
    }

    endMoveSelected(ev: any)
    {
        if(this.moveEvent) {
            ops.resumeEvent(this.moveEvent)
            ops.endUndoEvent(this.moveEvent)
        }
        this.moveEvent = '';
    }

    cancel()
    {
        if(this.activeTool != null)
        {
            this.activeTool.cancel()
            this.activeTool = null
        }
    }
}

class UIControllerSingleton
{
    private static instance: UIController
    constructor() {
        if(!UIControllerSingleton.instance) {
            UIControllerSingleton.instance = new UIController()
        }
    }

    getInstance() {
        return UIControllerSingleton.instance
    }
}

module.exports = UIControllerSingleton