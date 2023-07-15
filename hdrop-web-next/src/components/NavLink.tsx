'use client'

import Link from "next/link"
import { usePathname } from "next/navigation"

type Props = {
    href: string
    children: React.ReactNode
    reload?: boolean
}

export default function NavLink({ href, children, reload = false }: Props) {
    const pathName = usePathname()
    const isActive = pathName === href

    const inactiveClassName = 'text-gray-400'
    const activeClassName = 'text-gray-100'
    const extraClassName = isActive ? activeClassName : inactiveClassName

    const handleClick = (e: React.MouseEvent<HTMLAnchorElement, MouseEvent>) => {
        if (isActive && reload) {
            window.location.reload()
            e.preventDefault()
        }
    }

    return (
        <Link onClick={handleClick} className={`flex items-center h-full px-4 first:pl-0 last:pr-0 hover:text-gray-200 ${extraClassName}`} href={href}>{children}</Link>
    )
}
