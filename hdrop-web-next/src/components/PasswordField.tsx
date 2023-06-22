import { useState } from "react"

type Props = {
    onSubmit: (value: string) => void
}

export default function PasswordField({ onSubmit }: Props) {
    const [value, setValue] = useState<string>('')

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setValue(e.target.value)
    }

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            onSubmit(value)
        }
    }

    return (
        <input
            className="px-8 py-4 p-8 text-lg w-10/12 md:w-8/12 lg:w-1/2 xl:w-4/12 2xl:w-4/12 outline-none rounded-md bg-[hsl(0,0%,85%)] drop-shadow-xl text-black"
            placeholder="Password"
            type="password"
            value={value}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            autoFocus
        />
    )
}
