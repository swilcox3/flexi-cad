const gui = require('./gui')
import * as math from '../utils/math'
import { openSync } from 'fs';
import * as ops from '../operations/operations'

interface Tool
{
    onMouseDown(pt: math.Point3d, objId?:string):undefined,
    onMouseMove(pt: math.Point3d, objId?:string):undefined,
    cancel():undefined,
    finish():undefined
}

class UIController
{
    private activeTool: Tool
    private selectedObjs: Array<BABYLON.AbstractMesh>
    constructor() {
        this.activeTool = null
        this.selectedObjs = []
    }

    getSelectedObjs()
    {
        return this.selectedObjs
    }

    setActiveTool(tool:Tool)
    {
        if(this.activeTool != null)
        {
            this.activeTool.cancel()
        }
        this.activeTool = tool
    }

    leftClick(pt:math.Point3d, mesh: BABYLON.AbstractMesh)
    {
        if(this.activeTool != null)
        {
            var id = undefined;
            if(mesh != null) {
                id = mesh.name;
            }
            this.activeTool.onMouseDown(pt, id)
        }
        else if(mesh != null)
        {
            var mat = mesh.material as BABYLON.StandardMaterial;
            mat.diffuseColor = BABYLON.Color3.Green();
            this.selectedObjs.push(mesh)
        }
        else if(mesh == null)
        {
            this.selectedObjs.forEach((obj) => {
                var mat = obj.material as BABYLON.StandardMaterial;
                mat.diffuseColor = BABYLON.Color3.Gray();
            })
            this.selectedObjs = [];
        }
    }

    rightClick(pt:math.Point3d)
    {
        this.activeToolComplete()
    }

    mouseMove(pt:math.Point3d)
    {
        if(this.activeTool != null)
        {
            this.activeTool.onMouseMove(pt)
        }
    }

    activeToolComplete()
    {
        if(this.activeTool != null)
        {
            this.activeTool.finish()
            this.activeTool = null
        }
    }

    deleteSelected()
    {
        var event = ops.beginUndoEvent("Delete objs");
        this.selectedObjs.forEach((obj) => {
            ops.deleteObject(event, obj.name)
        });
        ops.endUndoEvent(event);
        this.selectedObjs = []
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