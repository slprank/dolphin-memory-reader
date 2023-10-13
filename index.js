const { memoryNew, memoryRead, memoryReadString } = require("./index.node");

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
class DolphinMemory {
  constructor() {
    this.memory = memoryNew();
  }

  read(address, length) {
    return memoryRead.call(this.memory, address, length);
  }
  // readString(address, chars) {
  //   return memoryReadString.call(this.memory, address, chars);
  // }
}

module.exports = DolphinMemory;
