import { Inter } from 'next/font/google'

import Header from '@/components/Header'
import Footer from '@/components/Footer'

import './globals.css'

const inter = Inter({ subsets: ['latin'] })

export const metadata = {
  title: 'hdrop',
  description: 'Secure file sharing for the modern age.',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={`grid grid-cols-[1fr] grid-rows-[auto_1fr_auto] ${inter.className}`}>
        <Header />
        {children}
        <Footer />
      </body>
    </html>
  )
}
