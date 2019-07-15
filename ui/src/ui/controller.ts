const gui = require('./gui')
import * as math from '../utils/math'

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
    private selectedObjs: Array<string>
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

    leftClick(pt:math.Point3d, mesh: string)
    {
        if(this.activeTool != null)
        {
            this.activeTool.onMouseDown(pt, mesh)
        }
        else if(mesh != null)
        {
            this.selectedObjs.push(mesh)
        }
        else if(mesh == null)
        {
            this.selectedObjs.forEach((mesh) => {

            })
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