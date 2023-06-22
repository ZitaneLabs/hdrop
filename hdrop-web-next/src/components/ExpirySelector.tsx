'use client'

import { useState } from "react"
import { Loader2 } from "lucide-react"

import { ApiClient } from "@/api"
import { toast } from "react-hot-toast"

type Props = {
    accessToken: string
    updateToken: string
    steps: { [key: string]: number }
}

export default function ExpirySelector({ accessToken, updateToken, steps }: Props) {
    const [currentExpiry, setCurrentExpiry] = useState(Object.values(steps).slice(-1)[0])
    const [loadingExpiry, setLoadingExpiry] = useState(0)

    const onClick = (value: number) => {
        // Optimistic update
        setLoadingExpiry(value)
        setCurrentExpiry(value)

        // Server update
        const promise = ApiClient.updateExpiry(accessToken, updateToken, value).then(() => {
            setLoadingExpiry(0)
            setCurrentExpiry(value)
        })
        toast.promise(promise, {
            loading: 'Updating expiry...',
            success: <b>Expiry updated!</b>,
            error: <b>Failed to update expiry!</b>
        })
    }

    return (
        <div className="flex h-10 justify-center items-center rounded-md overflow-clip">
            {Object.entries(steps).map(([key, value]) => {
                const isActive = currentExpiry === value
                const isLoading = loadingExpiry === value
                const inactiveClassName = 'bg-[hsl(0,0%,25%)]'
                const activeClassName = 'bg-[hsl(0,0%,40%)] hover:bg-[hsl(0,0%,40%)]'
                const extraClassName = isActive ? activeClassName : inactiveClassName
                const baseClassName = "relative flex w-full items-center justify-center gap-2 cursor-pointer px-6 py-2 hover:bg-[hsl(0,0%,30%)] transition-all"
                const className = [baseClassName, extraClassName].join(" ").trim()

                return (
                    <div className={className} key={key} onClick={() => onClick(value)}>
                        <div>{key}</div>
                        {isLoading && (
                            <Loader2 size={14} className="absolute animate-spin top-2 right-2" />
                        )}
                    </div>
                )
            })}
        </div>
    )
}
