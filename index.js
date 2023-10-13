const { memoryNew, memoryRead } = require("./index.node");

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
class DolphinMemory {
  constructor() {
    this.memory = memoryNew();
  }

  read(address, byteSize = 8) {
    return memoryRead.call(this.memory, address, byteSize);
  }
  // readString(address, chars) {
  //   return memoryReadString.call(this.memory, address, chars);
  // }
}

module.exports = DolphinMemory;
