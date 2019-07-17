import * as BABYLONGUI from "babylonjs-gui"
import {WallTool} from './tools/wall_tool'
import { openSync } from "fs";
import * as ops from '../operations/operations'
import { stringify } from "querystring";


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
        var button1 = BABYLONGUI.Button.CreateSimpleButton("but1", "Wall");
        button1.width = 1.0;
        button1.height = "40px";
        button1.color = "white";
        button1.cornerRadius = 20;
        button1.background = "green";
        button1.onPointerUpObservable.add(function () {
            var tool = new WallTool()
            mySingleton.setActiveTool(tool)
        });
        this.buttonPanel.addControl(button1);
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

    createWallOverlay(walls: Set<BABYLON.Mesh>) {
        var wallPanel = new BABYLONGUI.Grid();
        wallPanel.width = 1;
        wallPanel.height = "200px";
        wallPanel.addColumnDefinition(.5);
        wallPanel.addColumnDefinition(.5);
        this.objOverlay.addControl(wallPanel);
        var curRow = 0;
        var ids: Array<string> = [];
        var width:number = null;
        var height:number = null;
        walls.forEach((obj)=> {
            ids.push(obj.name)
            if(width == null) {
                width = obj.metadata.width;
            }
            else if (width != obj.metadata.width) {
                width = undefined;
            }
            if(height == null) {
                height = obj.metadata.height;
            }
            else if (height != obj.metadata.height) {
                height = undefined;
            }
        });
        var widthLabel = "";
        var heightLabel = "";
        if(width != null && width != undefined) {
            widthLabel = width.toString();
        }
        if(height != null && width != undefined) {
            heightLabel = height.toString();
        }
        wallPanel.addRowDefinition(40, true);
        this.createPropPair(wallPanel, curRow, ids, "Width", widthLabel);
        curRow = curRow + 1;
        wallPanel.addRowDefinition(40, true);
        this.createPropPair(wallPanel, curRow, ids, "Height", heightLabel);
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
        if(type == "Wall" && allSame) {
            this.createWallOverlay(data);
        }
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

