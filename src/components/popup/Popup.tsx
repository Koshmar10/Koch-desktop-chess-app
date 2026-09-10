import { type ReactNode } from 'react'

interface PopupProps {
  open: boolean
  children?: ReactNode
}

const Popup = ({ open, children }: PopupProps) => {
  if (!open) return null
  return (
    <div className='absolute inset-0 z-1000 flex h-[100vh] w-[100vw] items-center justify-center bg-white/20'>
      {children}
    </div>
  )
}

export default Popup
