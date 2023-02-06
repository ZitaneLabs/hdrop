const extensionToLanguage = {
    'c': 'c',
    'conf': 'ini',
    'config': 'ini',
    'cc': 'cpp',
    'cpp': 'cpp',
    'cs': 'csharp',
    'css': 'css',
    'csv': 'csv',
    'dart': 'dart',
    'go': 'go',
    'h': 'cpp',
    'hpp': 'cpp',
    'htm': 'html',
    'html': 'html',
    'ini': 'ini',
    'java': 'java',
    'js': 'javascript',
    'json': 'json',
    'json5': 'json5',
    'jsonp': 'jsonp',
    'jsx': 'jsx',
    'kt': 'kotlin',
    'less': 'less',
    'md': 'markdown',
    'php': 'php',
    'properties': 'ini',
    'py': 'python',
    'rb': 'ruby',
    'rs': 'rust',
    'sass': 'sass',
    'scss': 'scss',
    'sh': 'bash',
    'bash': 'bash',
    'sql': 'sql',
    'styl': 'stylus',
    'swift': 'swift',
    'toml': 'toml',
    'ts': 'typescript',
    'tsv': 'tsv',
    'tsx': 'tsx',
    'txt': 'text',
    'xml': 'xml',
    'yaml': 'yaml',
    'yml': 'yaml',
}

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
    text() {
        return new TextDecoder().decode(this.fileData)
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
        return this.guessMimeType().startsWith('video/')
    }

    isImage() {
        return this.guessMimeType().startsWith('image/')
    }

    isAudio() {
        return this.guessMimeType().startsWith('audio/')
    }

    isText() {
        return this.guessMimeType().startsWith('text/')
    }

    guessProgrammingLanguage() {
        // Get language by extension
        const language = extensionToLanguage[this.extension()]

        // If a language was found, return it
        if (language) return language

        // Otherwise, return a generic binary mime type
        return 'text'
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
        if (mimeType) return mimeType

        // If the file is a text file, return a generic text mime type
        if (extensionToLanguage[this.extension()]) { return 'text/plain' }

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