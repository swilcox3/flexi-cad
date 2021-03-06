import * as ops from '../../operations/operations'
import { Point3d, Vector3d } from "../../utils/math"
import * as BABYLON from 'babylonjs'

export class DoorTool {
    curTemp: JsDoor
    width: number
    height: number
    length: number
    undoEventId: string

    constructor(width = 1, height = 4, length = 3) {
        console.log("door tool constructor")
        this.curTemp = null;
        this.width = width;
        this.height = height;
        this.length = length;
        this.undoEventId = ''
    }

    canJoinToWall(hovered: BABYLON.Mesh) {
        return hovered && hovered.metadata && hovered.metadata.type == "Wall";
    }

    createDoor(pt: Point3d, picked: BABYLON.Mesh) {
        if (!this.undoEventId) {
            this.undoEventId = ops.beginUndoEvent("Create Door")
        }
        var door = new ops.dataModel.JsDoor(this.curTemp.first_pt(), this.curTemp.second_pt(), this.width, this.height)
        ops.deleteTempObject(this.curTemp.id())
        ops.createObj(this.undoEventId, door)
        if (this.canJoinToWall(picked)) {
            ops.snapToLine(this.undoEventId, picked.name, door.id(), pt)
        }
    }

    onMouseDown(pt: Point3d, picked: BABYLON.Mesh) {
        console.log("door down: ", pt);
        this.createDoor(new Point3d(pt.x, pt.y, 0), picked);
        this.curTemp = null;
    }

    onMouseMove(pt: Point3d, hovered: BABYLON.Mesh) {
        const joinable = this.canJoinToWall(hovered);
        if (this.curTemp == null) {
            var first = new Point3d(pt.x, pt.y, 0)
            var second = new Point3d(pt.x + this.length, pt.y, 0)
            this.curTemp = new ops.dataModel.JsDoor(first, second, this.width, this.height);
            ops.renderTempObject(this.curTemp)
        }
        else {
            if (joinable) {
                var first_promise = ops.getObjectData(hovered.name, "First");
                var second_promise = ops.getObjectData(hovered.name, "Second");
                Promise.all([first_promise, second_promise])
                    .then(([first, second]) => {
                        var project = ops.dataModel.projectOnLine(first, second, new Point3d(pt.x, pt.y, 0));
                        this.curTemp.set_first_pt(project);
                        //@ts-ignore
                        this.curTemp.setDir(new Vector3d(second.x - first.x, second.y - first.y, 0));
                    });
            }
            else {
                //@ts-ignore
                this.curTemp.set_first_pt(new Point3d(pt.x, pt.y, 0));
                //@ts-ignore
                this.curTemp.set_second_pt(new Point3d(pt.x + this.length, pt.y, 0));
            }
            this.drawDoor()
        }
        return joinable;
    }

    cancel() {
        if (this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
        ops.deleteTempObject(this.curTemp.id())
    }

    drawDoor() {
        if (this.curTemp) {
            ops.renderTempObject(this.curTemp)
        }
    }

    finish(pt: Point3d, picked: BABYLON.Mesh) {
        if (this.curTemp) {
            this.createDoor(new Point3d(pt.x, pt.y, 0), picked);
        }
        if (this.undoEventId) {
            ops.endUndoEvent(this.undoEventId)
        }
    }
}
