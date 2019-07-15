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

    onMouseDown(pt: math.Point3d)
    {
        if(this.firstPt == null)
        {
            this.firstPt = new math.Point3d(pt.x, pt.y, 0)
            this.secondPt = new math.Point3d(pt.x + 1, pt.y + 1, 0)
            this.activeTempId = ops.renderTempWall(this.firstPt, this.secondPt, this.activeWidth, this.activeHeight)
        }
        else
        {
            if(!this.undoEventId) {
                this.undoEventId = ops.beginUndoEvent("Create Wall")
            }
            ops.createWall(this.undoEventId, this.firstPt, this.secondPt, this.activeWidth, this.activeHeight, this.activeTempId)
            if(this.lastId) {
                ops.joinWalls(this.undoEventId, this.lastId, this.activeTempId, this.firstPt)
            }
            this.lastId = this.activeTempId;
            this.firstPt = new math.Point3d(pt.x, pt.y, 0)
            this.secondPt = new math.Point3d(pt.x + .1, pt.y + .1, 0)
            this.activeTempId = ops.renderTempWall(this.firstPt, this.secondPt, this.activeWidth, this.activeHeight)
        }
    }

    onMouseMove(pt: math.Point3d)
    {
        if(this.firstPt != null)
        {
            this.secondPt.x = pt.x
            this.secondPt.y = pt.y
            this.drawWall()
        }
    }

    cancel()
    {
        ops.deleteTempObject(this.activeTempId)
    }

    drawWall()
    {
        ops.renderTempWall(this.firstPt, this.secondPt, this.activeWidth, this.activeHeight, this.activeTempId)
    }

    finish()
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Wall")
        }
        ops.createWall(this.undoEventId, this.firstPt, this.secondPt, this.activeWidth, this.activeHeight, this.activeTempId)
        if(this.lastId) {
            ops.joinWalls(this.undoEventId, this.lastId, this.activeTempId, this.firstPt)
        }
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
