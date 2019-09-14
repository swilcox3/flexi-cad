console.log("Made it mouse 0");
import * as math from '../utils/math'
console.log("Made it mouse 1");
var uiController = require('./controller')
console.log("Made it mouse 2");
var uiSingleton = new uiController().getInstance()
console.log("Made it mouse 3");

function getGroundPosition(scene: BABYLON.Scene, ground: BABYLON.Mesh) 
{
    // Use a predicate to get position on the ground
    var pickinfo = scene.pick(scene.pointerX, scene.pointerY, mesh => { return mesh == ground });
    if (pickinfo.hit) {
        return pickinfo.pickedPoint;
    }
    return null;
}

function onPointerClick(scene: BABYLON.Scene, evt: MouseEvent, ground: BABYLON.Mesh) 
{
    var pickInfo = scene.pick(scene.pointerX, scene.pointerY);
    if (pickInfo.hit) {
        var currentMesh = pickInfo.pickedMesh;
        if(currentMesh == ground)
        {
            currentMesh = null
        }

        var currentPoint = getGroundPosition(scene, ground);
        if (evt.button == 0) {
            if (currentPoint) {
                uiSingleton.leftClick(math.transformGraphicToModelCoords(currentPoint), currentMesh)
            }
        }
        if (evt.button == 2) {
            if (currentPoint) {
                uiSingleton.rightClick(math.transformGraphicToModelCoords(currentPoint), currentMesh)
            }
        }
    }
}

function onPointerMove(scene: BABYLON.Scene, ground: BABYLON.Mesh, hovered: BABYLON.Mesh) {
    var current = getGroundPosition(scene, ground);
    if (!current) {
        return true;
    }

    return uiSingleton.mouseMove(math.transformGraphicToModelCoords(current), hovered)
}

module.exports = {
    onPointerClick,
    onPointerMove
}