export declare function getPid(): number | null;
export declare function getMemoryBaseAddress(pid: number | null): number | null;
export declare function getMemoryAddressSize(pid: number | null): number | null;

export declare function readMemory(pid: number, address: number, length: number): number[];
export declare function readMemoryWithDataSize(pid: number, address: number, length: number, dataTypeSize: DataTypeSize): number[];