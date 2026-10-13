import { Toaster as Sonner, type ToasterProps } from 'sonner'
import { useTheme } from '@/theme/ThemeContext'

export function Toaster(props: ToasterProps) {
  const { theme } = useTheme()

  return (
    <Sonner
      theme={theme}
      position="top-right"
      richColors
      closeButton
      {...props}
    />
  )
}
