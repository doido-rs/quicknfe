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
import { useUnsavedNavigationBlocker } from '@/hooks/useUnsavedFormGuard'

export function UnsavedChangesDialog() {
  const blocker = useUnsavedNavigationBlocker()

  if (blocker.status !== 'blocked') {
    return null
  }

  return (
    <AlertDialog open onOpenChange={(open) => !open && blocker.reset()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Alterações não salvas</AlertDialogTitle>
          <AlertDialogDescription>
            Existem alterações que ainda não foram salvas. Deseja sair e descartar as alterações?
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={blocker.reset}>Continuar editando</AlertDialogCancel>
          <AlertDialogAction
            onClick={() => {
              blocker.proceed()
            }}
          >
            Descartar e sair
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
