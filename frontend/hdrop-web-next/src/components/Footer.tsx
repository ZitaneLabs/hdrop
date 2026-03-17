import Wave from "react-wavify"

export default function Footer() {
    return (
        <footer>
            <div className="relative w-full h-16">
                <div className="absolute top-0 bottom-0 left-0 w-full flex justify-center z-50 pt-7 text-sm">
                    <a className="text-gray-300 hover:text-gray-100 cursor-pointer" href="https://github.com/ZitaneLabs/hdrop" target="_blank" rel="noopener noreferrer">Fork hdrop on GitHub</a>
                </div>
                <div className="absolute top-0 bottom-0 left-0 w-full z-0">
                    <Wave
                        fill='hsl(0,0%,25%)'
                        paused={false}
                        options={{
                            height: 1,
                            amplitude: 10,
                            speed: 0.25,
                            points: 3,
                        }}
                    />
                </div>
            </div>
        </footer>
    )
}
