import * as BABYLONGUI from "babylonjs-gui"
import {WallTool} from './tools/wall_tool'

export default class GUI
{
    private advancedTexture: BABYLONGUI.AdvancedDynamicTexture

    constructor()
    {
        this.advancedTexture = null
    }

    init()
    {
        var myController = require('./controller')
        var mySingleton = new myController().getInstance()
        this.advancedTexture = BABYLONGUI.AdvancedDynamicTexture.CreateFullscreenUI("ui1");
        var panel = new BABYLONGUI.StackPanel();
        panel.width = 0.5;
        panel.horizontalAlignment = BABYLONGUI.Control.HORIZONTAL_ALIGNMENT_LEFT;
        this.advancedTexture.addControl(panel);
        var button1 = BABYLONGUI.Button.CreateSimpleButton("but1", "Wall");
        button1.width = 0.5;
        button1.height = "40px";
        button1.color = "white";
        button1.cornerRadius = 20;
        button1.background = "green";
        button1.onPointerUpObservable.add(function () {
            var tool = new WallTool()
            mySingleton.setActiveTool(tool)
        });
        panel.addControl(button1);
    }

    createObjectOverlay(obj: any)
    {
        /*var panel = new BABYLONGUI.StackPanel()
        panel.width = 1.0
        panel.horizontalAlignment = BABYLONGUI.Control.HORIZONTAL_ALIGNMENT_LEFT;
        panel.left = 100
        this.advancedTexture.addControl(panel);*/
        for(var prop in obj)
        {
            console.log(prop)
            console.log(obj[prop])
            /*if(obj.hasOwnProperty(prop)) {
                var text = new BABYLONGUI.TextBlock()
                text.text = prop + ": " + JSON.stringify(obj[prop])
                text.color = "black"
                text.fontSize = 22
                panel.addControl(text)
            }*/
        }
    }
}

var guiInstance = new GUI()

module.exports = {
    guiInstance
}

