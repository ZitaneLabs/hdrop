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
    const mimeType = useMemo(() => {
        if (fileName === null) return ''
        return mime.getType(fileName.toLowerCase()) ?? 'application/octet-stream'
    }, [fileName])

    // Get mime type prefix (e.g. 'image', 'video', 'audio', etc.)
    const mimePrefix = mimeType.split('/')[0]

    // Object URLs own browser resources, so create them only after React commits.
    useEffect(() => {
        if (data === undefined || fileName === null) {
            // eslint-disable-next-line react-hooks/set-state-in-effect -- synchronize the external resource snapshot
            setObjectUrl('')
            return
        }

        const url = URL.createObjectURL(new Blob([data], { type: mimeType }))
        setObjectUrl(url)

        return () => {
            URL.revokeObjectURL(url)
        }
    }, [data, fileName, mimeType])

    // Try to decode data as text
    const textData = useMemo(() => {
        if (data === undefined) return null

        if (data.byteLength <= 3 || data.byteLength > 100_000) {
            return null
        }

        try {
            return new TextDecoder().decode(data)
        } catch {
            return null
        }
    }, [data])

    // No data, no preview
    if (data === undefined || fileName === null) return null

    const isImage = mimePrefix === 'image'
    const isVideo = mimePrefix === 'video'
    const isAudio = mimePrefix === 'audio'
    const isText = !isImage && !isVideo && !isAudio && textData !== null

    return (
        <div className="flex flex-col justify-center items-center gap-4 p-2 w-full h-full max-h-full mb-2 overflow-hidden">
            {/* File name */}
            <div className="truncate">{fileName}</div>

            {/* Image preview */}
            {isImage && (
                <img className="object-scale-down rounded-md min-w-min max-w-full max-h-fit h-1/2 drop-shadow-xl" alt={fileName} src={objectUrl} />
            )}

            {/* Video preview */}
            {isVideo && (
                <video className="object-scale-down rounded-lg min-w-min max-w-full max-h-fit h-1/2 drop-shadow-xl" src={objectUrl} controls />
            )}

            {/* Audio preview */}
            {isAudio && (
                <audio src={objectUrl} controls />
            )}

            {/* Text preview */}
            {isText && (
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
