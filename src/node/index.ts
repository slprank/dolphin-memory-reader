import { clearInterval } from "timers";
import { ByteSize } from "./types/enum";
import os from "os";

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
export default class DolphinMemory {
  memory: DolphinMemory | undefined;
  memoryNew: Function;
  memoryRead: Function;
  constructor() {
    this.memory = undefined;
    if (os.platform() === "win32") {
      const { memoryNew, memoryRead } = require("./index.node");
      this.memoryNew = memoryNew;
      this.memoryRead = memoryRead;
    } else {
      throw Error("Non Windows OS not supported");
    }
  }

  getIndexFile() { }

  async init() {
    if (this.memory) return;
    await new Promise((resolve) => {
      const interval = setInterval(() => {
        try {
          this.memory = this.memoryNew();
          if (this.memory) {
            resolve(undefined);
            clearInterval(interval);
          }
        } catch (err) {
          console.error(err);
        }
      }, 1000);
    });
  }

  read(address: number, byteSize: ByteSize = ByteSize.U8): number {
    if (!this.memory) throw new Error("Dolphin memory not initialized");
    return this.memoryRead.call(this.memory, address, byteSize);
  }

  readString(address: number, chars: number): string | undefined {
    const byteArray = [...Array(chars).keys()].map((i: number) =>
      this.read(address + ByteSize.U8 * i, ByteSize.U8)
    );
    const charArray = String.fromCharCode(...(byteArray as number[]));
    return charArray;
  }
}
