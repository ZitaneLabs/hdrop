'use client'

import { ReactNode, useState } from 'react'
import { Copy } from 'lucide-react'
import { toast } from 'react-hot-toast'

type Props = {
    value?: string
    children: ReactNode
}

export default function CopyButton({ value, children }: Props) {
    const onClick = () => {
        if (!value) return
        navigator.clipboard.writeText(value)
        toast.success('Copied to clipboard')
    }

    return (
        <div className="relative flex gap-2 px-2 py-[0.25em] items-center rounded-md bg-[hsl(0,0%,25%)] hover:bg-[hsl(0,0%,30%)] transition-all cursor-pointer" onClick={onClick}>
            <div className="text-sm">{children}</div>
            <div>
                <Copy size={14} />
            </div>
        </div>
    )
}
