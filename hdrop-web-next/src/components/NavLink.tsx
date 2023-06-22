'use client'

import Link from "next/link"
import { usePathname } from "next/navigation"

type Props = {
    href: string
    children: React.ReactNode
}

export default function NavLink({ href, children }: Props) {
    const pathName = usePathname()
    const isActive = pathName === href

    const inactiveClassName = 'text-gray-400'
    const activeClassName = 'text-gray-100'
    const extraClassName = isActive ? activeClassName : inactiveClassName

    return (
        <Link className={`flex items-center h-full px-4 first:pl-0 last:pr-0 hover:text-gray-200 ${extraClassName}`} href={href}>{children}</Link>
    )
}
