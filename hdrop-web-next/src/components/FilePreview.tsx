'use client'

/* eslint-disable @next/next/no-img-element */

import { useEffect, useMemo, useState } from "react"
import mime from 'mime/lite'

type Props = {
    data: ArrayBuffer | undefined
    fileName: string | null
}

export default function FilePreview({ data, fileName }: Props) {
    const [objectUrl, setObjectUrl] = useState('')
    const [mimeType, setMimeType] = useState('')

    // Get mime type prefix (e.g. 'image', 'video', 'audio', etc.)
    const mimePrefix = useMemo(() => mimeType.split('/')[0], [mimeType])

    // Try to decode data as text
    const textData = useMemo(() => {
        if (data === undefined) return null
        if (data.byteLength > 3 && data.byteLength <= 100_000) {
            try {
                const text = new TextDecoder().decode(data)
                return text
            } catch {
                return null
            }
        } else {
            return null
        }
    }, [data])

    // Prepare preview
    useEffect(() => {
        if (data === undefined || fileName === null) return

        // Detect and set mime type
        const mimeType = mime.getType(fileName.toLowerCase()) ?? 'application/octet-stream'
        setMimeType(mimeType)

        // Create object url
        const blob = new Blob([data], { type: mimeType })
        const objectUrl = URL.createObjectURL(blob)
        setObjectUrl(objectUrl)

        // Revoke object url when component unmounts
        return () => {
            URL.revokeObjectURL(objectUrl)
        }
    }, [data, fileName])

    // No data, no preview
    if (data === undefined || fileName === null) return null

    return (
        <div className="flex flex-col justify-center items-center gap-4 p-2 w-full h-full max-h-full mb-2 overflow-hidden">
            {/* File name */}
            <div className="truncate">{fileName}</div>

            {/* Image preview */}
            {mimePrefix === 'image' && (
                <img className="object-scale-down rounded-md min-w-min max-w-full max-h-fit h-1/2 drop-shadow-xl" alt={fileName} src={objectUrl} />
            )}

            {/* Video preview */}
            {mimePrefix === 'video' && (
                <video className="object-scale-down rounded-lg min-w-min max-w-full max-h-fit h-1/2 drop-shadow-xl" src={objectUrl} controls />
            )}

            {/* Audio preview */}
            {mimePrefix === 'audio' && (
                <audio src={objectUrl} controls />
            )}

            {/* Text preview */}
            {textData !== null && (
                <div className="bg-[hsl(0,0%,85%)] w-full md:w-10/12 lg:w-8/12 h-4/6 rounded-md drop-shadow-xl overflow-hidden">
                    <textarea className="px-4 py-2 w-full h-full outline-none bg-[hsl(0,0%,85%)] text-black font-mono flex-grow" readOnly value={textData} />
                </div>
            )}

            {/* Download button */}
            <div className="mt-2 px-16 py-4 bg-[hsl(0,0%,30%)] hover:bg-[hsl(0,0%,35%)] rounded-md cursor-pointer">
                <a href={objectUrl} download={fileName}>Download file</a>
            </div>
        </div>
    )
}
