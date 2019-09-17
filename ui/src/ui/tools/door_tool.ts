import {JsDoor, Point3d, dataModel} from '../../operations/operations'

export class DoorTool {
    curTemp: JsDoor
    width: number
    height: number
    length: number
    undoEventId: string

    constructor(width = 1, height = 4, length = 3)
    {
        this.curTemp = null;
        this.width = width;
        this.height = height;
        this.length = length;
        this.undoEventId = ''
    }

    canJoinToWall(hovered: BABYLON.Mesh) {
        return hovered && hovered.metadata && hovered.metadata.type == "Wall";
    }

    createDoor(pt: Point3d, picked: BABYLON.Mesh)
    {
        if(!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Door")
        }
        var door = new dataModel.JsDoor(this.curTemp.first_pt(), this.curTemp.second_pt(), this.width, this.height)
        ops.deleteTempObject(this.curTemp.id())
        ops.createObj(this.undoEventId, door)
        if(this.canJoinToWall(picked)) {
            ops.snapToLine(this.undoEventId, picked.name, door.id(), pt)
        }
    }

    onMouseDown(pt: Point3d, picked: BABYLON.Mesh)
    {
        this.createDoor(new dataModel.Point3d(pt.x, pt.y, 0), picked);
        this.curTemp = null;
    }

    onMouseMove(pt: Point3d, hovered: BABYLON.Mesh)
    {
        const joinable = this.canJoinToWall(hovered);
        if(this.curTemp == null)
        {
            var first = new dataModel.Point3d(pt.x, pt.y, 0)
            var second = new dataModel.Point3d(pt.x + this.length, pt.y, 0)
            this.curTemp = new dataModel.JsDoor(first, second, this.width, this.height);
            ops.renderTempObject(this.curTemp)
        }
        else
        {
            if(joinable) {
                var first_promise = ops.getObjectData(hovered.name, "First");
                var second_promise = ops.getObjectData(hovered.name, "Second");
                Promise.all([first_promise, second_promise])
                .then(([first, second]) => {
                    var project = dataModel.projectOnLine(first, second, new dataModel.Point3d(pt.x, pt.y, 0));
                    this.curTemp.set_first_pt(project);
                    this.curTemp.setDir(new dataModel.Vector3d(second.x - first.x, second.y - first.y, 0));
                });
            }
            else {
                this.curTemp.set_first_pt(new dataModel.Point3d(pt.x, pt.y, 0));
                this.curTemp.set_second_pt(new dataModel.Point3d(pt.x + this.length, pt.y, 0));
            }
            this.drawDoor()
        }
        return joinable;
    }

    cancel()
    {
        if(this.undoEventId) {
            ops.cancelEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.id)
    }

    drawDoor()
    {
        if(this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: Point3d, picked: BABYLON.Mesh)
    {
        if(this.curTemp) {
            this.createDoor(new dataModel.Point3d(pt.x, pt.y, 0), picked);
        }
        if(this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
