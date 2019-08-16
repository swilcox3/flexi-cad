import * as BABYLONGUI from "babylonjs-gui"
import {WallTool} from './tools/wall_tool'
import {DoorTool} from './tools/door_tool'
import {DimensionTool} from './tools/dimension_tool'
import * as ops from '../operations/operations'
import * as math from '../utils/math'
import { promises } from "fs";

function demoUnit(position: math.Point3d) 
{
    var ops = require("../operations/operations");
    var kernel = require("../../native/index.node");
    let sideLength = 50;
    let width = 1;
    let height = 5;
    let position_2 = new math.Point3d(position.x + sideLength, position.y, position.z);
    let position_3 = new math.Point3d(position.x + sideLength, position.y + sideLength, position.z);
    let position_4 = new math.Point3d(position.x, position.y + sideLength, position.z)
    let wall1 = new kernel.Wall(position, position_2, width, height)
    let wall2 = new kernel.Wall(position_2, position_3, width, height)
    let wall3 = new kernel.Wall(position_3, position_4, width, height)
    let wall4 = new kernel.Wall(position_4, position, width, height)
    let event = ops.beginUndoEvent("Demo");
    let wall_1_promise = ops.createObj(event, wall1);
    let wall_2_promise = ops.createObj(event, wall2);
    let wall_3_promise = ops.createObj(event, wall3);
    let wall_4_promise = ops.createObj(event, wall4);
    let promises = [];
    promises.push(Promise.all([wall_1_promise, wall_2_promise]).then(([mesh_1, mesh_2]) => {
        ops.joinAtPoints(event, mesh_1.name, mesh_2.name, position_2);
        return mesh_1.name;
    }));
    promises.push(Promise.all([wall_2_promise, wall_3_promise]).then(([mesh_2, mesh_3]) => {
        ops.joinAtPoints(event, mesh_2.name, mesh_3.name, position_3);
        return mesh_2.name;
    }));
    promises.push(Promise.all([wall_3_promise, wall_4_promise]).then(([mesh_3, mesh_4]) => {
        ops.joinAtPoints(event, mesh_3.name, mesh_4.name, position_4);
        return mesh_3.name;
    }));
    promises.push(Promise.all([wall_4_promise, wall_1_promise]).then(([mesh_4, mesh_1]) => {
        ops.joinAtPoints(event, mesh_4.name, mesh_1.name, position);
        return mesh_4.name;
    }));

    Promise.all(promises).then(([id_1, id_2, id_3, id_4]) => {
        let door_pos = new math.Point3d(position.x + sideLength/2, position.y, position.z);
        let door = new kernel.Door(door_pos, new math.Point3d(door_pos.x + 5, door_pos.y, door_pos.z), 1, 4);
        ops.createObj(event, door).then((door_mesh: BABYLON.Mesh) => {
            ops.snapToLine(event, id_1, door_mesh.name, door_pos);
        });
        let offset = 2;
        let dim_1 = new kernel.Dimension(position, position_2, offset);
        let dim_2 = new kernel.Dimension(position_2, position_3, offset);
        let dim_3 = new kernel.Dimension(position_3, position_4, offset);
        let dim_4 = new kernel.Dimension(position_4, position, offset);
        ops.createObj(event, dim_1).then((dim_1_mesh: BABYLON.Mesh) => {
            ops.snapToPoint(event, dim_1_mesh.name, id_1, position);
            ops.snapToPoint(event, dim_1_mesh.name, id_1, position_2);
        });
        ops.createObj(event, dim_2).then((dim_2_mesh: BABYLON.Mesh) => {
            ops.snapToPoint(event, dim_2_mesh.name, id_2, position_2);
            ops.snapToPoint(event, dim_2_mesh.name, id_2, position_3);
        });
        ops.createObj(event, dim_3).then((dim_3_mesh: BABYLON.Mesh) => {
            ops.snapToPoint(event, dim_3_mesh.name, id_3, position_3);
            ops.snapToPoint(event, dim_3_mesh.name, id_3, position_4);
        });
        ops.createObj(event, dim_4).then((dim_4_mesh: BABYLON.Mesh) => {
            ops.snapToPoint(event, dim_4_mesh.name, id_4, position_4);
            ops.snapToPoint(event, dim_4_mesh.name, id_4, position);
        });
    });
}

export default class GUI
{
    private advancedTexture: BABYLONGUI.AdvancedDynamicTexture
    private buttonPanel: BABYLONGUI.StackPanel;
    private objOverlay: BABYLONGUI.StackPanel;

    constructor()
    {
        this.advancedTexture = null
        this.buttonPanel = null
        this.objOverlay = null
    }

    newButton(name: string, label: string, callback: ()=>void) {
        var button = BABYLONGUI.Button.CreateSimpleButton(name, label);
        button.width = 1.0;
        button.height = "40px";
        button.color = "white";
        button.cornerRadius = 20;
        button.background = "green";
        button.onPointerUpObservable.add(callback);
        this.buttonPanel.addControl(button);
    }

    init()
    {
        var myController = require('./controller')
        var mySingleton = new myController().getInstance()
        this.advancedTexture = BABYLONGUI.AdvancedDynamicTexture.CreateFullscreenUI("ui1");
        this.buttonPanel = new BABYLONGUI.StackPanel();
        this.buttonPanel.width = "100px";
        this.buttonPanel.horizontalAlignment = BABYLONGUI.Control.HORIZONTAL_ALIGNMENT_LEFT;
        this.advancedTexture.addControl(this.buttonPanel);
        this.objOverlay = new BABYLONGUI.StackPanel();
        this.objOverlay.width = "300px";
        this.objOverlay.horizontalAlignment = BABYLONGUI.Control.HORIZONTAL_ALIGNMENT_RIGHT;
        this.advancedTexture.addControl(this.objOverlay);
        this.newButton("but1", "Wall", () => {
            var tool = new WallTool()
            mySingleton.setActiveTool(tool)
        });
        this.newButton("but2", "Door", () => {
            var tool = new DoorTool()
            mySingleton.setActiveTool(tool)
        })
        this.newButton("but3", "Dimension", () => {
            var tool = new DimensionTool()
            mySingleton.setActiveTool(tool)
        })
        this.newButton("demo", "Demo 1", () => {
           demoUnit(new math.Point3d(0, 0, 0)) 
        })
        this.newButton("demo 2", "Demo 100", () => {
            var position = new math.Point3d(0, 0, 0);
            for(let i = 0; i < 10; i++) {
                for(let j = 0; j < 10; j++) {
                    demoUnit(new math.Point3d(position.x + 75*i, position.y + 75*j, 0)) 
                }
            }
        })
    }

    createPropPair(parent: BABYLONGUI.Grid, curRow: number, objIds:Array<string>, label:string, value:string) {
        var text = new BABYLONGUI.TextBlock();
        text.text = label
        text.color = "black"
        text.height = "40px"
        text.width = 1
        parent.addControl(text, curRow, 0);
        var edit = new BABYLONGUI.InputText();
        edit.text = value;
        edit.background = "white"
        edit.color = "black"
        edit.focusedBackground = "grey"
        edit.height = "40px"
        edit.width = 1
        edit.metadata = label
        edit.onBlurObservable.add((evt) => {
            var event = ops.beginUndoEvent("prop set");
            var data = {[evt.metadata]: Number(evt.text)};
            if(objIds.length == 1) {
                ops.setObjectData(event, objIds[0], data);
            }
            else {
                var dataArray: Array<[string, any]> = [];
                objIds.forEach((id: string) => {
                    dataArray.push([id, data])
                })
                ops.setObjectsDatas(event, dataArray)
            }
            ops.endUndoEvent(event)
        })
        parent.addControl(edit, curRow, 1);
    }

    populateObjectOverlay(objs: Set<BABYLON.Mesh>) {
        var objPanel = new BABYLONGUI.Grid();
        objPanel.width = 1;
        objPanel.height = "200px";
        objPanel.addColumnDefinition(.5);
        objPanel.addColumnDefinition(.5);
        this.objOverlay.addControl(objPanel);
        var curRow = 0;
        var ids: Array<string> = [];
        var props: any = {};
        objs.forEach((obj)=> {
            ids.push(obj.name)
            for (var property in obj.metadata) {
                if(obj.metadata.hasOwnProperty(property) && property !== "type") {
                    if(props[property] === undefined) {
                        props[property] = obj.metadata[property];
                    }
                    else if (props[property] !== null && props[property] !== obj.metadata[property]) {
                        props[property] = null;
                    }
                }
            }
        });
        for (const prop of Object.keys(props)) {
            objPanel.addRowDefinition(40, true);
            this.createPropPair(objPanel, curRow, ids, prop, props[prop].toString());
            curRow = curRow + 1;
        }
    }

    setObjectOverlay(data: Set<BABYLON.Mesh>)
    {
        this.objOverlay.clearControls();
        var type:string = null;
        var countObjs = 0;
        var allSame = true;
        data.forEach((obj) => {
            if(type == null) {
                type = obj.metadata.type;
            }
            else if(type != obj.metadata.type) {
                allSame = false;
            }
            countObjs = countObjs + 1;
        });
        var labelText = "";
        if(allSame) {
            if(countObjs > 1) {
                labelText = countObjs.toString() + " " + type + "s";
            }
            else {
                labelText = type;
            }
        }
        else {
            labelText = countObjs.toString() + " Objects"
        }
        var label = new BABYLONGUI.TextBlock();
        label.text = labelText;
        label.color = "black"
        label.height = "40px"
        label.width = 1
        this.objOverlay.addControl(label);
        this.populateObjectOverlay(data);
    }
    
    clearObjectOverlay() 
    {
        this.objOverlay.clearControls();
    }
}

var guiInstance = new GUI()

module.exports = {
    guiInstance
}

