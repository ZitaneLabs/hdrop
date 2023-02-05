export default class DecryptedFileInfo {
    /**
     * Construct a new `DecryptedFileInfo`.
     * 
     * @param {ArrayBuffer} fileData
     * @param {string} fileName
     */
    constructor(fileData, fileName) {
        this.fileData = fileData
        this.fileName = fileName
    }

    /**
     * @returns {ArrayBuffer}
     */
    data() {
        return this.fileData
    }

    /**
     * @returns {string}
     */
    name() {
        return this.fileName
    }

    /**
     * @returns {string}
     */
    extension() {
        return this.fileName.split('.').pop()
    }

    isVideo() {
        return [
            'flv',
            'mp4',
            'mkv',
            'mov',
            'webm',
        ].includes(this.extension())
    }

    isImage() {
        return [
            'png',
            'jpg',
            'jpeg',
            'gif',
            'tiff',
            'webp',
        ].includes(this.extension())
    }

    isAudio() {
        return [
            'mp3',
            'mpeg',
            'wav',
            'ogg',
            'flac',
        ].includes(this.extension())
    }

    guessMimeType() {
        const extensionToMimeType = {
            // Audio
            'mp3': 'audio/mpeg',
            'mpeg': 'audio/mpeg',
            'wav': 'audio/wav',
            'ogg': 'audio/ogg',
            'flac': 'audio/flac',
            // Video
            'flv': 'video/x-flv',
            'mp4': 'video/mp4',
            'mkv': 'video/x-matroska',
            'mov': 'video/quicktime',
            'webm': 'video/webm',
            // Image
            'png': 'image/png',
            'jpg': 'image/jpeg',
            'jpeg': 'image/jpeg',
            'gif': 'image/gif',
            'tiff': 'image/tiff',
            'webp': 'image/webp',
        }

        // Get mime type by extension
        const mimeType = extensionToMimeType[this.extension()]

        // If a mime type was found, return it
        if (mimeType) {
            return mimeType
        }

        // Otherwise, return a generic binary mime type
        return 'application/octet-stream'
    }

    objectUrl() {
        return URL.createObjectURL(
            new Blob(
                [this.fileData],
                {
                    type: this.guessMimeType()
                }
            )
        )
    }
}