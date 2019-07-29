import * as ops from '../../operations/operations'
import * as math from '../../utils/math'

export class WallTool {
    firstPt: math.Point3d
    secondPt: math.Point3d
    activeWidth: number
    activeHeight: number
    lastId: string
    activeTempId: string
    undoEventId: string

    constructor(width = 1, height = 5)
    {
        this.firstPt = null
        this.secondPt = null
        this.activeWidth = width 
        this.activeHeight = height
        this.lastId = ''
        this.activeTempId = ''
        this.undoEventId = ''
    }

    canJoinToWall(hovered: BABYLON.Mesh) {
        return hovered && hovered.metadata.type == "Wall" && hovered.name != this.lastId && hovered.name != this.activeTempId;
    }

    createWall(picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Wall")
        }
        ops.createWall(this.undoEventId, this.firstPt, this.secondPt, this.activeWidth, this.activeHeight, this.activeTempId)
        if(this.lastId) {
            ops.joinAtPoint(this.undoEventId, this.lastId, this.activeTempId, this.firstPt)
        }
        if(this.canJoinToWall(picked)) {
            ops.joinAtPoint(this.undoEventId, this.activeTempId, picked.name, this.secondPt);
        }
    }

    onMouseDown(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        if(this.firstPt == null)
        {
            this.firstPt = new math.Point3d(pt.x, pt.y, 0)
            this.secondPt = new math.Point3d(pt.x + 1, pt.y + 1, 0)
            this.activeTempId = ops.renderTempWall(this.firstPt, this.secondPt, this.activeWidth, this.activeHeight)
        }
        else
        {
            this.createWall(picked);
            this.lastId = this.activeTempId;
            this.firstPt = new math.Point3d(pt.x, pt.y, 0)
            this.secondPt = new math.Point3d(pt.x + .1, pt.y + .1, 0)
            this.activeTempId = ops.renderTempWall(this.firstPt, this.secondPt, this.activeWidth, this.activeHeight)
        }
    }

    onMouseMove(pt: math.Point3d, hovered: BABYLON.Mesh)
    {
        if(this.firstPt != null)
        {
            this.secondPt.x = pt.x
            this.secondPt.y = pt.y
            this.drawWall()
        }
        return this.canJoinToWall(hovered);
    }

    cancel()
    {
        if(this.undoEventId) {
            ops.cancelEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.activeTempId)
    }

    drawWall()
    {
        ops.renderTempWall(this.firstPt, this.secondPt, this.activeWidth, this.activeHeight, this.activeTempId)
    }

    finish(pt: math.Point3d, picked: BABYLON.Mesh)
    {
        this.createWall(picked);
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
