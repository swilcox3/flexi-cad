import { app, BrowserWindow, Menu } from "electron";
const { dialog } = require('electron');
import * as path from "path";
import * as url from "url";

let windows: Map<string, Electron.BrowserWindow> = new Map();
let curWindow: Electron.BrowserWindow;
let defaultNew: string = "defaultNew.flx";

function createWindow(title: string) {
  var newWindow = new BrowserWindow(
    {
      "title": title,
      show: false,
      webPreferences: {
        nodeIntegration: true
      }
    });

  newWindow.loadURL(url.format({
    pathname: path.join(__dirname, "../../index_electron.html"),
    protocol: "file:",
    slashes: true,
  }));

  newWindow.webContents.openDevTools();

  newWindow.once("ready-to-show", () => {
    newWindow.webContents.send("newFile");
    newWindow.show();
  });
  newWindow.on("closed", () => {
    windows.delete(title)
  });
  newWindow.on("focus", () => {
    curWindow = newWindow;
  });
  windows.set(title, newWindow);
  curWindow = newWindow;
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on("ready", () => {
  createWindow("Tech Demo");
});

// Quit when all windows are closed.
app.on("window-all-closed", () => {
  // On OS X it is common for applications and their menu bar
  // to stay active until the user quits explicitly with Cmd + Q
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  // On OS X it"s common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (windows.size === 0) {
    createWindow("Tech Demo");
  }
});
