import NavLink from "./NavLink"

export default function Header() {
    return (
        <header className="flex h-16 justify-center items-center backdrop-blur-md">
            <div className="flex z-10 px-6 h-full w-full md:max-w-md items-center justify-between">
                <div>
                    <NavLink reload href="/">{process.env.NEXT_PUBLIC_APP_NAME ?? "hdrop"}</NavLink>
                </div>
                <nav className="flex items-center h-full">
                    <NavLink reload href="/">Upload</NavLink>
                    <NavLink href="/privacy">Privacy</NavLink>
                </nav>
            </div>
        </header>
    )
}
