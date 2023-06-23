import { ReactNode } from 'react'
import { Trash } from 'lucide-react'
import { ApiClient } from '@/api'
import { toast } from 'react-hot-toast'
import { useRouter } from 'next/navigation'

type Props = {
    accessToken: string
    updateToken: string
    children: ReactNode
}

export default function DeleteButton({ accessToken, updateToken, children }: Props) {
    const router = useRouter()

    const onClick = () => {
        toast.promise(
            ApiClient.deleteFile(accessToken, updateToken).then(() => {
                setTimeout(() => {
                    window.location.reload()
                }, 500)
            }),
            {
                loading: 'Deleting...',
                success: <b>File deleted!</b>,
                error: <b>Deletion failed!</b>
            }
        )
    }

    return (
        <div className="flex gap-2 px-2 py-[0.25em] items-center rounded-md bg-[hsl(0,75%,30%)] hover:bg-[hsl(0,75%,35%)] transition-all cursor-pointer" onClick={onClick}>
            <div className="text-sm">{children}</div>
            <Trash size={14} />
        </div>
    )
}
