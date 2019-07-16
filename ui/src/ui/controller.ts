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

    deselect()
    {
        this.selectedObjs.forEach((obj) => {
            var mat = obj.material as BABYLON.StandardMaterial;
            mat.diffuseColor = BABYLON.Color3.Gray();
        })
        this.selectedObjs = [];
    }

    leftClick(pt:math.Point3d, mesh: BABYLON.Mesh)
    {
        if(this.activeTool != null)
        {
            this.activeTool.onMouseDown(pt, mesh)
        }
        else if(mesh != null)
        {
            this.deselect();
            var mat = mesh.material as BABYLON.StandardMaterial;
            mat.diffuseColor = BABYLON.Color3.Green();
            this.selectedObjs.push(mesh)
        }
        else if(mesh == null)
        {
            this.deselect();
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
            this.deselect();
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