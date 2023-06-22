'use client'

import { useCallback, useMemo, useState } from 'react'
import { useDropzone } from 'react-dropzone'
import { Copy, Lock, Upload } from 'lucide-react'
import { toast, Toaster } from 'react-hot-toast'
import Wave from 'react-wavify'

import APIClient from '@/api/ApiClient'
import Uploader, { UploadPhase, UploadResult } from '@/api/Uploader'
import { Switch, Match as UntypedMatch, CopyButton, DeleteButton, ExpirySelector } from '@/components'

const Match = UntypedMatch<UploadPhase | null>

export default function Home() {
  const [phase, setPhase] = useState<UploadPhase | null>(null)
  const [progress, setProgress] = useState<number>(0)
  const [uploadResult, setUploadResult] = useState<UploadResult | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Compute the download URL without password
  const downloadUrl = useMemo(() => {
    if (uploadResult) {
      return APIClient.getDownloadLink(uploadResult.accessToken)
    }
  }, [uploadResult])

  // Compute the download URL with password
  const downloadUrlWithPassword = useMemo(() => {
    if (uploadResult) {
      return APIClient.getDownloadLink(uploadResult.accessToken, uploadResult.password)
    }
  }, [uploadResult])

  const onProgressChange = (phase: UploadPhase, progress: number) => {
    setPhase(phase)
    setProgress(progress)
  }

  const onUploadComplete = (uploadResult: UploadResult) => {
    setUploadResult(uploadResult)
  }

  const onDrop = useCallback((files: File[]) => {
    const file = files[0]
    Uploader.uploadFile(file, onProgressChange, onUploadComplete).catch(e => {
      console.error(e)
      setError(e.message)
    })
  }, [])

  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop })

  return (
    <main className="flex flex-col justify-center items-center">
      <Toaster containerStyle={{ marginTop: '4rem' }} toastOptions={{
        style: {
          background: 'hsl(0,0%,30%)',
          color: 'white',
        }
      }} />
      <Switch value={phase}>

        {/* Waiting for file upload */}
        <Match on={null}>
          <div className="flex flex-col gap-4 justify-center items-center bg-[hsla(0,0%,0%,.1)] p-8 w-10/12 h-1/2 md:w-8/12 lg:w-1/2 xl:w-4/12 2xl:w-4/12 rounded-md drop-shadow-md cursor-pointer md:hover:scale-105 transition ease-out duration-[.5s] backdrop-blur-sm" {...getRootProps()}>
            <Upload size={64} color="white" className={isDragActive ? 'transition-all animate-bounce' : 'transition-all'} />
            <span className="text-center">
              {isDragActive ? 'Drop the file here ...' : 'Drag and drop a file here, or click to select one'}
            </span>
            <input {...getInputProps()} />
          </div>
        </Match>

        {/* File encryption */}
        <Match on='encrypting'>
          <div className="flex flex-col gap-4 relative justify-center items-center w-64 h-64 rounded-full bg-[hsla(0,0%,0%,.1)] animate-pulse overflow-hidden select-none">
            <Lock size={48} />
            <span className="z-10">Encrypting...</span>
          </div>
        </Match>

        {/* File upload */}
        <Match on='uploading'>
          <div className="relative flex justify-center items-center w-64 h-64 rounded-full bg-[hsla(0,0%,0%,.1)] overflow-hidden select-none">
            <span className="text-gray-300 text-xl">Uploading...</span>
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

        {/* Done */}
        <Match on='done'>
          <div className="flex flex-col gap-4">
            <ExpirySelector
              accessToken={uploadResult?.accessToken ?? ''}
              updateToken={uploadResult?.updateToken ?? ''}
              steps={{
                '15m': 60 * 15,
                '1h': 60 * 60,
                '6h': 60 * 60 * 6,
                '12h': 60 * 60 * 12,
                '24h': 60 * 60 * 24
              }}
            />
            <div className="flex flex-col gap-8 pt-8 justify-center items-center bg-[hsla(0,0%,0%,.1)] rounded-lg drop-shadow-md transition ease-out duration-[.5s] backdrop-blur-sm overflow-hidden">
              <div className="flex w-[calc(100%_-_4rem)] gap-4 justify-center items-center px-4 py-4 bg-[hsl(0,0%,10%)] rounded-md cursor-pointer" onClick={() => {
                navigator.clipboard.writeText(downloadUrl ?? '')
                toast.success('Copied to clipboard')
              }}>
                <span className="font-mono">{downloadUrl?.replace(/https?:[/]{2}/, '')}</span>
                <Copy size={20} />
              </div>
              <div className="flex w-full gap-2 px-8 py-6 bg-[hsl(0,0%,15%)]">
                <CopyButton value={uploadResult?.password}>Password</CopyButton>
                <CopyButton value={downloadUrlWithPassword}>Link with Password</CopyButton>
                <DeleteButton
                  accessToken={uploadResult?.accessToken ?? ''}
                  updateToken={uploadResult?.updateToken ?? ''}>
                  Delete
                </DeleteButton>
              </div>
            </div>
          </div>
        </Match>
      </Switch>
    </main>
  )
}
