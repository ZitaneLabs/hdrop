import { ReactNode } from 'react'
import { Trash } from 'lucide-react'

type Props = {
    children: ReactNode
}

export default function DeleteButton({ children }: Props) {
    return (
        <div className="flex gap-2 px-2 py-[0.25em] items-center rounded-md bg-[hsl(0,75%,30%)] hover:bg-[hsl(0,75%,35%)] transition-all cursor-pointer">
            <div className="text-sm">{children}</div>
            <Trash size={14} />
        </div>
    )
}
