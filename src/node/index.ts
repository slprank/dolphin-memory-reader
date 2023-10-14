import { ByteSize } from "./types/enum";

const { memoryNew, memoryRead } = require("./index.node");

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
export default class DolphinMemory {
  memory;
  constructor() {
    this.memory = memoryNew();
  }

  read(address: number, byteSize: ByteSize = ByteSize.U8) {
    console.log("Reading:", address, byteSize);
    return memoryRead.call(this.memory, address, byteSize);
  }
  // readString(address, chars) {
  //   return memoryReadString.call(this.memory, address, chars);
  // }
}
