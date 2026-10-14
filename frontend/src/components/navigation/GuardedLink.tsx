import { Link, useNavigate, type LinkProps } from '@tanstack/react-router'
import { useGuardedNavigate } from '@/hooks/useUnsavedFormGuard'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

type GuardedLinkProps = LinkProps & {
  children: React.ReactNode
  className?: string
}

export function GuardedLink({ children, onClick, ...props }: GuardedLinkProps) {
  const navigate = useNavigate()
  const { requestNavigation, pendingConfirmation, confirmDiscard, cancelNavigation } =
    useGuardedNavigate()

  return (
    <>
      <Link
        {...props}
        onClick={(e) => {
          e.preventDefault()
          requestNavigation(() => {
            void navigate({
              to: props.to,
              params: props.params,
              search: props.search,
            })
          })
          onClick?.(e)
        }}
      >
        {children}
      </Link>
      <AlertDialog open={pendingConfirmation} onOpenChange={(open) => !open && cancelNavigation()}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Alterações não salvas</AlertDialogTitle>
            <AlertDialogDescription>
              Deseja sair sem salvar? As alterações serão perdidas.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={cancelNavigation}>Continuar editando</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDiscard}>Descartar e sair</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
