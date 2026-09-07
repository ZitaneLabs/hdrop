import { Children, cloneElement, isValidElement, ReactNode } from "react"

type Props<T> = {
    value: T
    children: ReactNode
}

export default function Switch<T>({ value, children }: Props<T>) {
    return (
        <div className="relative flex justify-center items-center w-full h-full">
            {Children.map(children, child => {
                if (isValidElement(child) && child.type === Match<T>) {
                    return cloneElement(child, { active: (child.props as MatchProps<T>).on === value } as MatchProps<T>)
                }
            })}
        </div>
    )
}

type MatchProps<T> = {
    on: T
    when?: boolean
    children: ReactNode
    active?: boolean
}

export function Match<T>({ on, when = true, children, active: baseActive = false }: MatchProps<T>) {
    const active = baseActive && when
    const baseClassName = "absolute w-full h-full flex flex-col justify-center items-center transition-all duration-500"
    const activeClassName = ""
    const inactiveClassName = "opacity-0 pointer-events-none"
    const className = [baseClassName, active ? activeClassName : inactiveClassName].join(" ")

    return (
        <div className={className}>
            {children}
        </div>
    )
}
