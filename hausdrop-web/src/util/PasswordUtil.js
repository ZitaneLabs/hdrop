class PasswordUtil {
    static generateSecurePassword() {
        const wordCount = 4
        return Array(wordCount).fill().map(() => PasswordUtil.generateSecureWord()).join('')
    }

    static generateSecureWord() {
        const length = 6
        const charset = "abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ123456789"
        return Array(length).fill().map(() => {
            return charset[Math.floor(Math.random() * charset.length)]
        }).join('')
    }
}

export default PasswordUtil