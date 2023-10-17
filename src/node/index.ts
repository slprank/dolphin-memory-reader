import { clearInterval } from "timers";
import { ByteSize } from "./types/enum";

const { memoryNew, memoryRead } = require("./index.node");

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
export default class DolphinMemory {
  memory: DolphinMemory | undefined;
  constructor() {
    this.memory = undefined;
  }

  async init() {
    if (this.memory) return;
    await new Promise((resolve) => {
      const interval = setInterval(() => {
        this.memory = memoryNew();
        if (this.memory) {
          resolve(undefined);
          clearInterval(interval);
        }
      }, 1000);
    });
  }

  read(address: number, byteSize: ByteSize = ByteSize.U8): number {
    try {
      if (!this.memory) throw new Error("Dolphin memory not initialized");
      return memoryRead.call(this.memory, address, byteSize);
    } catch (err) {
      console.error(err);
    }
    throw new Error("Cannot read from memory");
  }

  readString(address: number, chars: number) {
    const byteArray = [...Array(chars).keys()].map((i: number) =>
      this.read(address + ByteSize.U8 * i, ByteSize.U8)
    );
    const charArray = String.fromCharCode(...byteArray);
    return charArray;
  }
}
