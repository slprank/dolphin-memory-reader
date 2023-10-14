import { ByteSize } from "./types/enum";

export = DolphinMemory;

declare class DolphinMemory {
  constructor();

  read(address: number, byteSize: ByteSize): Uint8Array;
  readString(address: number, chars: number): string;
}
