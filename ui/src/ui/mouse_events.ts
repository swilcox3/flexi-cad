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

function onPointerDown(scene: BABYLON.Scene, canvas: HTMLCanvasElement, evt: MouseEvent, ground: BABYLON.Mesh, camera: BABYLON.Camera) 
{
    var uiSingleton = new uiController().getInstance()
    if (evt.button == 0) {
        // check if we are under a mesh
        var pickInfo = scene.pick(scene.pointerX, scene.pointerY);
        if (pickInfo.hit) {
            var currentMesh = pickInfo.pickedMesh;
            if(currentMesh == ground)
            {
                currentMesh = null
            }

            if(!currentPoint)
            {
                setTimeout(function () {
                    camera.detachControl(canvas);
                }, 0);
            }
            currentPoint = getGroundPosition(scene, ground);

            if (currentPoint) {
                uiSingleton.leftClick(math.transformGraphicToModelCoords(currentPoint), currentMesh)
            }
        }
    }
    if (evt.button == 2) {
        if (currentPoint) {
            uiSingleton.rightClick(currentPoint)
            camera.attachControl(canvas, true);
            currentPoint = null;
            return;
        }
    }
}

function onPointerMove(scene: BABYLON.Scene, ground: BABYLON.Mesh) {
    var uiSingleton = new uiController().getInstance()
    if (!currentPoint) {
        return;
    }

    var current = getGroundPosition(scene, ground);
    if (!current) {
        return;
    }

    uiSingleton.mouseMove(math.transformGraphicToModelCoords(current))
    currentPoint = current;
}

module.exports = {
    onPointerDown,
    onPointerMove
}