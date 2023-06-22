'use client'

import { useEffect, useMemo, useState } from "react"
import { useParams } from "next/navigation"
import { toast, Toaster } from "react-hot-toast"
import { Cpu, Key } from "lucide-react"
import Wave from "react-wavify"

import { Downloader, DownloadPhase, DownloadResult } from "@/api"
import { Switch, Match as UntypedMatch, PasswordField, FilePreview } from "@/components"

const Match = UntypedMatch<DownloadPhase | null>

export default function DownloadFilePage() {
    const { accessToken } = useParams()

    const getPasswordFromHash = () => {
        if (window.location.hash.length === 0) return null
        return window.location.hash.slice(1)
    }

    const [hashPassword, setHashPassword] = useState<string | null | undefined>(undefined)
    const [userPassword, setUserPassword] = useState<string | null>(null)
    const [phase, setPhase] = useState<DownloadPhase | null>(null)
    const [progress, setProgress] = useState<number>(0)
    const [fileName, setFileName] = useState<string | null>(null)
    const [result, setResult] = useState<DownloadResult | null>(null)

    const password = useMemo(() => {
        return hashPassword ?? userPassword
    }, [hashPassword, userPassword])

    useEffect(() => {
        setHashPassword(getPasswordFromHash())
    }, [])

    useEffect(() => {
        if (accessToken === undefined || password === null) return
        Downloader.downloadFile({
            accessToken,
            password,
            onProgressChange(phase: DownloadPhase, progress: number) {
                setPhase(phase)
                setProgress(progress)
            },
            onFileNameObtained(fileName: string) {
                setFileName(fileName)
            },
            onDownloadComplete(result: DownloadResult) {
                setResult(result)
            }
        }).catch(e => {
            console.error(e)
            toast.error(e.message, {
                duration: 10000,
            })
        })
    }, [accessToken, password])

    return (
        <main className="flex flex-col justify-center items-center">
            <Toaster containerStyle={{ marginTop: '4rem' }} toastOptions={{
                style: {
                background: 'hsl(0,0%,30%)',
                color: 'white',
                }
            }} />
            <Switch value={phase}>

                {/* Wait for password input */}
                <Match on={null} when={hashPassword === null}>
                    <PasswordField onSubmit={value => setUserPassword(value)} />
                </Match>

                {/*  */}
                <Match on='validating'>
                    <div className="flex flex-col gap-4 relative justify-center items-center w-64 h-64 rounded-full bg-[hsla(0,0%,0%,.1)] animate-pulse overflow-hidden select-none">
                        <Key size={48} />
                        <span className="z-10">Authorizing...</span>
                    </div>
                </Match>
                <Match on='downloading'>
                    <div className="relative flex justify-center items-center w-64 h-64 rounded-full bg-[hsla(0,0%,0%,.1)] overflow-hidden select-none">
                        <span className="text-gray-300 text-xl">Downloading...</span>
                        <div className='absolute w-full h-full transition-all duration-500 ease-linear' style={{ bottom: `calc(-${1 - progress} * 16rem)` }}>
                        <Wave
                            className="h-full mix-blend-color-dodge"
                            fill='hsl(0,0%,30%)'
                            paused={false}
                            options={{
                                height: 0,
                                amplitude: 5,
                                speed: 0.5,
                                points: 3,
                            }}
                        />
                        </div>
                    </div>
                </Match>
                <Match on='decrypting'>
                    <div className="flex flex-col gap-4 relative justify-center items-center w-64 h-64 rounded-full bg-[hsla(0,0%,0%,.1)] animate-pulse overflow-hidden select-none">
                        <Cpu size={48} />
                        <span className="z-10">Decrypting...</span>
                    </div>
                </Match>
                <Match on='done'>
                    <FilePreview data={result?.data} fileName={fileName ?? ''} />
                </Match>
            </Switch>
        </main>
    )
}
