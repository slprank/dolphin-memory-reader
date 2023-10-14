import { ByteSize } from "./types/enum";
export default class DolphinMemory {
    memory: any;
    constructor();
    read(address: number, byteSize?: ByteSize): any;
}
