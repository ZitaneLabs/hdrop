const ALPHABET = [
	"A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
	"N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
	"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m",
	"n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
	"0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "+", "/"
]

class Base64Util {
    /**
     * Encodes a byte array into a base64 string.
     * 
     * @param {Uint8Array} bytes
     */
    static encode(bytes) {
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
            result += "=="
        }
        if (i === l) { // 2 octets yet to write
            result += ALPHABET[bytes[i - 2] >> 2]
            result += ALPHABET[((bytes[i - 2] & 0x03) << 4) | (bytes[i - 1] >> 4)]
            result += ALPHABET[(bytes[i - 1] & 0x0F) << 2]
            result += "="
        }
        return result
    }
}

export default Base64Util