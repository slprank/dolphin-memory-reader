"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const enum_1 = require("./types/enum");
const { memoryNew, memoryRead } = require("../../index.node");
// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
class DolphinMemory {
    constructor() {
        this.memory = memoryNew();
    }
    read(address, byteSize = enum_1.ByteSize.U8) {
        return memoryRead.call(this.memory, address, byteSize);
    }
}
exports.default = DolphinMemory;
