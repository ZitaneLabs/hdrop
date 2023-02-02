export default class MobileDeviceUtil {
    static isMobileDevice() {
        const reUserAgents = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i
        const isMobileUA = reUserAgents.test(navigator.userAgent)
        const hasManyTouchPoints = navigator.maxTouchPoints > 10
        const isMobile = isMobileUA || hasManyTouchPoints
        return isMobile
    }
}