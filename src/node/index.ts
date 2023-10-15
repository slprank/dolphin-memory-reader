import { ByteSize } from "./types/enum";

const { memoryNew, memoryRead } = require("./index.node");

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
export default class DolphinMemory {
  memory;
  constructor() {
    try {
      this.memory = memoryNew();
    } catch (err) {
      console.error(err);
    }
  }

  read(address: number, byteSize: ByteSize = ByteSize.U8): number {
    try {
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
