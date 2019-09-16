import {JsWall, Point3d, dataModelWasm} from '../../operations/operations'

export class WallTool {
    curTemp: JsWall;
    width: number
    height: number
    lastId: string
    undoEventId: string

    constructor(width = 1, height = 5)
    {
        this.curTemp = null;
        this.width = width;
        this.height = height;
        this.lastId = ''
        this.undoEventId = ''
    }

    canJoinToWall(hovered: BABYLON.Mesh) {
        return this.curTemp && hovered && hovered.metadata && hovered.metadata.type == "Wall" && hovered.name != this.lastId && hovered.name != this.curTemp.id;
    }

    createWall(picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Wall")
        }
        var wall = new dataModelWasm.JsWall(this.curTemp.first_pt, this.curTemp.second_pt, this.width, this.height);
        ops.deleteTempObject(this.curTemp.id);
        ops.createObj(this.undoEventId, wall)
        if(this.lastId) {
            ops.joinAtPoints(this.undoEventId, this.lastId, wall.id, wall.first_pt)
        }
        if(this.canJoinToWall(picked)) {
            ops.joinAtPoints(this.undoEventId, picked.name, wall.id, wall.second_pt);
        }
        this.lastId = wall.id;
    }

    onMouseDown(pt: Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp == null)
        {
            var first = new dataModelWasm.Point3d(pt.x, pt.y, 0)
            var second = new dataModelWasm.Point3d(pt.x + 1, pt.y + 1, 0)
            this.curTemp = new dataModelWasm.JsWall(first, second, this.width, this.height);
            ops.renderTempObject(this.curTemp)
        }
        else
        {
            this.createWall(picked);
            var first = new dataModelWasm.Point3d(pt.x, pt.y, 0)
            var second = new dataModelWasm.Point3d(pt.x + .1, pt.y + .1, 0)
            this.curTemp = new dataModelWasm.JsWall(first, second, this.width, this.height);
            ops.renderTempObject(this.curTemp)
        }
    }

    onMouseMove(pt: Point3d, hovered: BABYLON.Mesh)
    {
        if(this.curTemp != null)
        {
            this.curTemp.second_pt = new dataModelWasm.Point3d(pt.x, pt.y, 0);
            this.drawWall()
        }
        return this.canJoinToWall(hovered);
    }

    cancel()
    {
        if(this.undoEventId) {
            ops.cancelEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.id)
    }

    drawWall()
    {
        if(this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: Point3d, picked: BABYLON.Mesh)
    {
        this.createWall(picked);
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
