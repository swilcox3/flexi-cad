import * as BABYLONGUI from "babylonjs-gui"
import {WallTool} from './tools/wall_tool'

export default class GUI
{
    private advancedTexture: BABYLONGUI.AdvancedDynamicTexture
    private buttonPanel: BABYLONGUI.StackPanel;
    private objOverlay: BABYLONGUI.Grid;

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
        this.objOverlay = new BABYLONGUI.Grid();
        this.objOverlay.addColumnDefinition(0.5);
        this.objOverlay.addColumnDefinition(0.5);
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

    createObjectOverlay(obj: any)
    {
        var curRow = 0;
        for(var prop in obj)
        {
            this.objOverlay.addRowDefinition(40, true);
            var text = new BABYLONGUI.TextBlock();
            text.text = prop
            text.color = "black"
            text.height = "40px"
            text.width = 1
            this.objOverlay.addControl(text, curRow, 0);
            var edit = new BABYLONGUI.InputText();
            edit.text = JSON.stringify(obj[prop])
            edit.color = "black"
            edit.background = "white"
            edit.height = "40px"
            edit.width = 1
            this.objOverlay.addControl(edit, curRow, 1);
            curRow = curRow + 1;
        }
    }
}

var guiInstance = new GUI()

module.exports = {
    guiInstance
}

