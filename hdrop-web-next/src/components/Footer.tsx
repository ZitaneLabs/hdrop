import Wave from "react-wavify"

export default function Footer() {
    return (
        <footer>
            <div className="w-full h-16">
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
        </footer>
    )
}
