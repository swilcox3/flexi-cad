console.log("made it gui 0")
import * as BABYLONGUI from "babylonjs-gui"
console.log("made it gui 1")
import {WallTool} from './tools/wall_tool'
console.log("made it gui 2")
import {DoorTool} from './tools/door_tool'
console.log("made it gui 3")
import {DimensionTool} from './tools/dimension_tool'
console.log("made it gui 4")
import * as ops from '../operations/operations'
console.log("made it gui 5")
import {Point3d} from '../../data-model-wasm/pkg/data_model_wasm'
console.log("made it gui 6")

export default class GUI
{
    private advancedTexture: BABYLONGUI.AdvancedDynamicTexture
    private buttonPanel: BABYLONGUI.StackPanel;
    private connPanel: BABYLONGUI.StackPanel;
    private objOverlay: BABYLONGUI.StackPanel;

    constructor()
    {
        this.advancedTexture = null
        this.buttonPanel = null
        this.connPanel = null
        this.objOverlay = null
    }

    newButton(name: string, label: string, panel: BABYLONGUI.Container, callback: ()=>void) {
        var button = BABYLONGUI.Button.CreateSimpleButton(name, label);
        button.width = "100px";
        button.height = "40px";
        button.color = "white";
        button.cornerRadius = 20;
        button.background = "green";
        button.onPointerUpObservable.add(callback);
        panel.addControl(button);
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
        this.newButton("but1", "Wall", this.buttonPanel, () => {
            var tool = new WallTool()
            mySingleton.setActiveTool(tool)
        });
        this.newButton("but2", "Door", this.buttonPanel, () => {
            var tool = new DoorTool()
            mySingleton.setActiveTool(tool)
        })
        this.newButton("but3", "Dimension", this.buttonPanel, () => {
            var tool = new DimensionTool()
            mySingleton.setActiveTool(tool)
        })
        this.newButton("demo", "Demo 1", this.buttonPanel, () => {
            ops.demo(new Point3d(0, 0, 0)) 
        })
        this.newButton("demo 2", "Demo 100", this.buttonPanel, () => {
            ops.demo_100(new Point3d(0, 0, 0))
        });
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
            const event = ops.beginUndoEvent("prop set");
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
            for (var property in obj.metadata.obj) {
                if(obj.metadata.obj.hasOwnProperty(property)) {
                    if(props[property] === undefined) {
                        props[property] = obj.metadata.obj[property];
                    }
                    else if (props[property] !== null && props[property] !== obj.metadata.obj[property]) {
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

