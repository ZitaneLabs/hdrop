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

export function Match<T>({ when = true, children, active: baseActive = false }: MatchProps<T>) {
    const active = baseActive && when
    const baseClassName = "absolute w-full h-full flex flex-col justify-center items-center transition-opacity duration-500"
    const activeClassName = ""
    const inactiveClassName = ""
    const className = [baseClassName, active ? activeClassName : inactiveClassName].join(" ")
    const style = {
        opacity: active ? 1 : 0,
        visibility: active ? "visible" : "hidden",
        pointerEvents: active ? "auto" : "none",
    } as const

    return (
        <div className={className} style={style}>
            {children}
        </div>
    )
}
