import * as BABYLON from 'babylonjs'
import * as BABYLONGUI from "babylonjs-gui"
import * as TEXTURES from './babylonjs.proceduralTextures.min.js'
import * as mouse from '../ui/mouse_events'
import * as gui from '../ui/gui'
import { UIControllerSingleton } from '../ui/controller'

function getHoveredMesh(scene: BABYLON.Scene, ground: BABYLON.Mesh) {
    var pickinfo = scene.pick(scene.pointerX, scene.pointerY, mesh => { return mesh != ground });
    if (pickinfo.hit) {
        return pickinfo.pickedMesh;
    }
    return null;
}

export class Renderer {
    private _canvas: HTMLCanvasElement
    private _engine: BABYLON.Engine
    private _scene: BABYLON.Scene
    private _highlight: BABYLON.HighlightLayer

    createScene(canvas: HTMLCanvasElement, engine: BABYLON.Engine) {
        this._canvas = canvas;
        this._engine = engine;
        // This creates a basic Babylon Scene object (non-mesh)
        const scene = new BABYLON.Scene(engine);
        this._scene = scene;
        this._highlight = new BABYLON.HighlightLayer("highlight1", this._scene);
        // This creates and positions a free camera (non-mesh)
        const camera = new BABYLON.ArcRotateCamera("camera1", -Math.PI / 2, 1.0, 500, new BABYLON.Vector3(500, 0, -500), scene);
        camera.panningSensibility = 50;
        camera.panningInertia = .7;
        // This attaches the camera to the canvas
        camera.attachControl(canvas, true);
        // This creates a light, aiming 0,1,0 - to the sky (non-mesh)
        const light = new BABYLON.HemisphericLight("light", new BABYLON.Vector3(0, 1, 0), this._scene);
        // Default intensity is 1. Let's dim the light a small amount
        light.intensity = 0.7;
        light.parent = camera;

        var ground = BABYLON.Mesh.CreateGround("ground", 10000, 10000, 0, this._scene, false);
        var groundMaterial = new BABYLON.StandardMaterial("ground", this._scene);
        groundMaterial.specularColor = BABYLON.Color3.Black();
        ground.material = groundMaterial;

        gui.guiInstance.init();

        var onPointerClick = (evt: MouseEvent) => {
            mouse.onPointerClick(this._scene, evt, ground)
        }

        var current_hover: BABYLON.Mesh = null;
        var onPointerMove = (evt: MouseEvent) => {
            var hovered = getHoveredMesh(this._scene, ground)
            var layer = this._scene.getHighlightLayerByName("highlight1");
            if (current_hover && hovered != current_hover) {
                layer.removeMesh(current_hover)
            }
            if (mouse.onPointerMove(this._scene, ground, hovered as BABYLON.Mesh)) {
                if (hovered) {
                    layer.addMesh(hovered as BABYLON.Mesh, BABYLON.Color3.Green());
                    current_hover = hovered as BABYLON.Mesh;
                }
            }
        }

        this._scene.onPointerObservable.add((pointerInfo) => {
            switch (pointerInfo.type) {
                case BABYLON.PointerEventTypes.POINTERDOWN:
                    break;
                case BABYLON.PointerEventTypes.POINTERUP:
                    break;
                case BABYLON.PointerEventTypes.POINTERMOVE:
                    onPointerMove(pointerInfo.event)
                    break;
                case BABYLON.PointerEventTypes.POINTERWHEEL:
                    break;
                case BABYLON.PointerEventTypes.POINTERPICK:
                    break;
                case BABYLON.PointerEventTypes.POINTERTAP:
                    onPointerClick(pointerInfo.event)
                    break;
                case BABYLON.PointerEventTypes.POINTERDOUBLETAP:
                    break;
            }
        });
    }
    initialize(canvas: HTMLCanvasElement) {
        const engine = new BABYLON.Engine(canvas, true, { stencil: true });
        this.createScene(canvas, engine);
        engine.runRenderLoop(() => {
            this._scene.render();
        });
        window.addEventListener('resize', function () {
            engine.resize();
        });
    }

    stop() {
        this._engine.stopRenderLoop();
    }

    applyNewMeshProps(mesh: BABYLON.Mesh, temp?: boolean) {
        if (!temp) {
            var objMaterial = new BABYLON.StandardMaterial("obj", this._scene)
            /*var woodTexture = new TEXTURES.WoodProceduralTexture("wood", 1024, this._scene);
            woodTexture.ampScale = 80.0;
            woodTexture.woodColor = BABYLON.Color3.Red();
            objMaterial.diffuseTexture = woodTexture;*/
            objMaterial.diffuseColor = BABYLON.Color3.Gray();
            objMaterial.backFaceCulling = false;
            mesh.material = objMaterial;
            var pointerDragBehavior = new BABYLON.PointerDragBehavior({ dragPlaneNormal: new BABYLON.Vector3(0, 1, 0) });
            pointerDragBehavior.useObjectOrienationForDragging = false;
            var uiSingleton = new UIControllerSingleton().getInstance();
            pointerDragBehavior.onDragObservable.add((ev: any) => {
                uiSingleton.objDrag(ev, mesh)
            })
            pointerDragBehavior.onDragEndObservable.add((ev: any) => {
                uiSingleton.objDragEnd(ev)
            })
            pointerDragBehavior.moveAttached = false;
            mesh.addBehavior(pointerDragBehavior)
        }
        else {
            var objMaterial = new BABYLON.StandardMaterial("temp", this._scene);
            objMaterial.backFaceCulling = false;
            objMaterial.wireframe = true;
            mesh.material = objMaterial;
        }
    }

    renderMesh(triangles: any, id: string, temp?: boolean) {
        var mesh = this._scene.getMeshByName(id) as BABYLON.Mesh
        if (!mesh) {
            mesh = new BABYLON.Mesh(id, this._scene);
            this.applyNewMeshProps(mesh, temp);
        }
        mesh.metadata = triangles.metadata;
        var vertexData = new BABYLON.VertexData();
        vertexData.positions = triangles.positions;
        vertexData.indices = triangles.indices;
        vertexData.applyToMesh(mesh);
    }

    showNormals(mesh: BABYLON.Mesh) {
        var normals = mesh.getVerticesData(BABYLON.VertexBuffer.NormalKind);
        var positions = mesh.getVerticesData(BABYLON.VertexBuffer.PositionKind);
        var color = BABYLON.Color3.White();
        var size = 1;

        var lines = [];
        for (var i = 0; i < normals.length; i += 3) {
            var v1 = BABYLON.Vector3.FromArray(positions, i);
            var v2 = v1.add(BABYLON.Vector3.FromArray(normals, i).scaleInPlace(size));
            lines.push([v1.add(mesh.position), v2.add(mesh.position)]);
        }
        var normalLines = BABYLON.MeshBuilder.CreateLineSystem("normalLines", { lines: lines }, this._scene);
        normalLines.color = color;
        return normalLines;
    }

    renderObject(json: any, id: string, temp?: boolean) {
        var mesh = this._scene.getMeshByName(id) as BABYLON.Mesh
        switch (json.metadata.type) {
            case "Dimension":
                var first = new BABYLON.Vector3(json.first.x, 1, json.first.z);
                var first_off = new BABYLON.Vector3(json.first_off.x, 1, json.first_off.z);
                var second = new BABYLON.Vector3(json.second.x, 1, json.second.z);
                var second_off = new BABYLON.Vector3(json.second_off.x, 1, json.second_off.z);
                var text_pos = new BABYLON.Vector3(json.text_pos.x, 1, json.text_pos.z);

                if (!mesh) {
                    mesh = BABYLON.MeshBuilder.CreateSphere(id, { diameter: 1 }, this._scene);
                }
                mesh.metadata = json.metadata;
                mesh.position = text_pos;

                //Performance TANKS with this method of displaying text in 3D space.
                /*var plane = this._scene.getMeshByName(id + "_plane");
                if (!plane) {
                    plane = BABYLON.Mesh.CreatePlane(id + "_plane", 500, this._scene);
                    plane.rotation = new BABYLON.Vector3(Math.PI / 2, 0, 0);
                    plane.parent = mesh;
                    plane.position.y = 2;
                }
                var guiTexture = BABYLONGUI.AdvancedDynamicTexture.CreateForMesh(plane);
                var textLabel = new BABYLONGUI.TextBlock();
                textLabel.name = id + "_text";
                textLabel.text = json.text;
                textLabel.width = 1;
                textLabel.height = 1;
                textLabel.color = "black";
                guiTexture.addControl(textLabel);*/

                var line_1 = this._scene.getMeshByName(id + "_line1");
                if (!line_1) {
                    var line_1_pts = [first, first_off];
                    line_1 = BABYLON.MeshBuilder.CreateLines(id + "_line1", { points: line_1_pts, updatable: true }, this._scene)
                }
                else {
                    var positions = line_1.getVerticesData(BABYLON.VertexBuffer.PositionKind);
                    positions = [first.x, first.y, first.z, first_off.x, first_off.y, first_off.z]
                    line_1.updateVerticesData(BABYLON.VertexBuffer.PositionKind, positions);
                }

                var line_2 = this._scene.getMeshByName(id + "_line2");
                if (!line_2) {
                    var line_2_pts = [second, second_off];
                    line_2 = BABYLON.MeshBuilder.CreateLines(id + "_line2", { points: line_2_pts, updatable: true }, this._scene)
                }
                else {
                    var positions = line_2.getVerticesData(BABYLON.VertexBuffer.PositionKind);
                    positions = [second.x, second.y, second.z, second_off.x, second_off.y, second_off.z]
                    line_2.updateVerticesData(BABYLON.VertexBuffer.PositionKind, positions);
                }
                var line_3 = this._scene.getMeshByName(id + "_line3");
                if (!line_3) {
                    var line_3_pts = [first_off, second_off];
                    line_3 = BABYLON.MeshBuilder.CreateLines(id + "_line3", { points: line_3_pts, updatable: true }, this._scene)
                }
                else {
                    var positions = line_3.getVerticesData(BABYLON.VertexBuffer.PositionKind);
                    positions = [first_off.x, first_off.y, first_off.z, second_off.x, second_off.y, second_off.z]
                    line_3.updateVerticesData(BABYLON.VertexBuffer.PositionKind, positions);
                }
                break;
        }
    }

    deleteMesh(id: string) {
        var mesh = this._scene.getMeshByName(id)
        if (mesh) {
            mesh.dispose()
        }
        var mesh = this._scene.getMeshByName(id + "_line1")
        if (mesh) {
            mesh.dispose()
        }
        var mesh = this._scene.getMeshByName(id + "_line2")
        if (mesh) {
            mesh.dispose()
        }
        var mesh = this._scene.getMeshByName(id + "_line3")
        if (mesh) {
            mesh.dispose()
        }
    }

    getMesh(id: string) {
        return this._scene.getMeshByName(id) as BABYLON.Mesh
    }
}