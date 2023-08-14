const ALPHABET = [
	"A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
	"N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
	"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m",
	"n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
	"0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "+", "/"
]

const DECODE_LUT = [
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 62, 255, 255, 255, 63,
	52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 255, 255, 255, 0, 255, 255,
	255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
	15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 255, 255, 255, 255, 255,
	255, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
	41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51
];

export default class Base64 {
    private static decodeChar(charCode: number): number {
        if (charCode >= DECODE_LUT.length) {
            throw new Error("Unable to parse base64 string.");
        }
        const byte = DECODE_LUT[charCode];
        if (byte === 255) {
            throw new Error("Unable to parse base64 string.");
        }
        return byte;
    }

    private static determineMissingOctetCount(len: number): number {
        const remainder = len % 4
        if (remainder === 1) {
            throw new Error("Unable to parse base64 string.")
        }
        const lut = {
            0: 0,
            2: 2,
            3: 1,
        } as Record<number, number>
        return lut[remainder]
    }

    /** Decode a base64 string into a byte array. */
    static decode(str: string): Uint8Array {
        str = Base64.fromDisplay(str)
        const missingOctets = this.determineMissingOctetCount(str.length)
        const n = str.length + missingOctets
        const result = new Uint8Array(3 * (n / 4))
        for (let i = 0, j = 0; i < n; i += 4, j += 3) {
            const buffer =
                (Base64.decodeChar(str.charCodeAt(i)) << 18) |
                (Base64.decodeChar(str.charCodeAt(i + 1)) << 12) |
                (i + 2 < n ? Base64.decodeChar(str.charCodeAt(i + 2)) << 6 : 0) |
                (i + 3 < n ? Base64.decodeChar(str.charCodeAt(i + 3)) : 0)
            result[j] = buffer >> 16
            if (i + 2 < n) result[j + 1] = (buffer >> 8) & 0xFF
            if (i + 3 < n) result[j + 2] = buffer & 0xFF
        }
        return result.subarray(0, result.length - missingOctets)
    }

    /** Encode a byte array into a base64 string. */
    static encode(bytes: Uint8Array): string {
        let result = '', i, l = bytes.length
        for (i = 2; i < l; i += 3) {
            result += ALPHABET[bytes[i - 2] >> 2]
            result += ALPHABET[((bytes[i - 2] & 0x03) << 4) | (bytes[i - 1] >> 4)]
            result += ALPHABET[((bytes[i - 1] & 0x0F) << 2) | (bytes[i] >> 6)]
            result += ALPHABET[bytes[i] & 0x3F]
        }
        if (i === l + 1) { // 1 octet yet to write
            result += ALPHABET[bytes[i - 2] >> 2]
            result += ALPHABET[(bytes[i - 2] & 0x03) << 4]
        }
        if (i === l) { // 2 octets yet to write
            result += ALPHABET[bytes[i - 2] >> 2]
            result += ALPHABET[((bytes[i - 2] & 0x03) << 4) | (bytes[i - 1] >> 4)]
            result += ALPHABET[(bytes[i - 1] & 0x0F) << 2]
        }
        return Base64.toDisplay(result)
    }

    private static fromDisplay(text: string): string {
        return text.replaceAll('-', '+').replaceAll('_', '/').replaceAll('=', '')
    }

    private static toDisplay(base64: string): string {
        return base64.replaceAll('+', '-').replaceAll('/', '_')
    }
}
