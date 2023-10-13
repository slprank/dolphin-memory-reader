export enum ByteSize {
  U8 = 8,
  U16 = 16,
  U32 = 32,
}

export declare class DolphinMemory {
  constructor();

  read(address: number, byteSize: number = ByteSize.U8): Uint8Array;
  readString(address: number, chars: number): string;
}

