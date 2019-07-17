import * as math from '../utils/math'
var uiController = require('./controller')

var currentPoint: math.Point3d = null

function getGroundPosition(scene: BABYLON.Scene, ground: BABYLON.Mesh) 
{
    // Use a predicate to get position on the ground
    var pickinfo = scene.pick(scene.pointerX, scene.pointerY, mesh => { return mesh == ground });
    if (pickinfo.hit) {
        return pickinfo.pickedPoint;
    }
    return null;
}

function onPointerDown(scene: BABYLON.Scene, canvas: HTMLCanvasElement, evt: MouseEvent, ground: BABYLON.Mesh)
{
    var uiSingleton = new uiController().getInstance()
    var pickInfo = scene.pick(scene.pointerX, scene.pointerY);
    if (pickInfo.hit) {
        var currentMesh = pickInfo.pickedMesh;
        if(currentMesh == ground)
        {
            currentMesh = null
        }

        if (evt.button == 0 && currentMesh) {
            uiSingleton.leftDown(currentMesh)
        }
    }
} 

function onPointerClick(scene: BABYLON.Scene, canvas: HTMLCanvasElement, evt: MouseEvent, ground: BABYLON.Mesh) 
{
    var uiSingleton = new uiController().getInstance()
    var pickInfo = scene.pick(scene.pointerX, scene.pointerY);
    if (pickInfo.hit) {
        var currentMesh = pickInfo.pickedMesh;
        if(currentMesh == ground)
        {
            currentMesh = null
        }

        currentPoint = getGroundPosition(scene, ground);
        if (evt.button == 0) {
            if (currentPoint) {
                uiSingleton.leftClick(math.transformGraphicToModelCoords(currentPoint), currentMesh)
            }
        }
        if (evt.button == 2) {
            if (currentPoint) {
                uiSingleton.rightClick(currentPoint, currentMesh)
                currentPoint = null;
                currentMesh = null;
            }
        }
    }
}

function onPointerMove(scene: BABYLON.Scene, ground: BABYLON.Mesh, hovered: BABYLON.Mesh) {
    var uiSingleton = new uiController().getInstance()
    if (!currentPoint) {
        return true;
    }

    var current = getGroundPosition(scene, ground);
    if (!current) {
        return true;
    }

    currentPoint = current;
    return uiSingleton.mouseMove(math.transformGraphicToModelCoords(current), hovered)
}

module.exports = {
    onPointerDown,
    onPointerClick,
    onPointerMove
}